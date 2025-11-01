//! Virtual pool and graduation logic.
//!
//! Implements a Meteora DBC-style virtual reserves pool with automatic
//! graduation to a full DAMM v2 pool when the bonding curve is exhausted.

use serde::{Deserialize, Serialize};

use crate::curve::{BondingCurve, CurveError, CurveType};
use crate::math::{FeeBreakdown, FeeSchedule, PriceImpact, SlippageGuard};

// ─── Pool Types ──────────────────────────────────────────────────────────────

/// A virtual reserves pool for Meteora DBC-style token launches.
///
/// Virtual reserves are synthetic liquidity used during the bonding curve
/// phase. They do not represent real deposited tokens — instead, the curve
/// math determines pricing. Real liquidity only exists after graduation.
///
/// ```text
///   Launch Phase (DBC)              Post-Graduation (DAMM v2)
///   ┌─────────────────┐             ┌─────────────────────┐
///   │  Virtual Pool   │  graduate   │   Real AMM Pool     │
///   │  bonding curve  │ ──────────▶ │   constant-product  │
///   │  price via math │             │   real reserves     │
///   └─────────────────┘             └─────────────────────┘
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPool {
    /// Pool identifier (e.g., mint address bytes)
    pub pool_id: [u8; 32],

    /// Virtual SOL reserve (lamports)
    pub virtual_sol_reserve: u64,

    /// Virtual token reserve
    pub virtual_token_reserve: u64,

    /// Real SOL collected from buys (lamports)
    pub real_sol_reserve: u64,

    /// Real tokens remaining in the pool
    pub real_token_reserve: u64,

    /// The bonding curve governing price discovery
    pub curve: BondingCurve,

    /// Fee schedule for swaps
    pub fee_schedule: FeeSchedule,

    /// Graduation configuration
    pub graduation: GraduationConfig,

    /// Current pool state
    pub state: PoolState,

    /// Total volume traded through this pool (lamports)
    pub total_volume: u64,

    /// Number of trades executed
    pub trade_count: u64,
}

/// Pool state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolState {
    /// Pool is active and accepting swaps via the bonding curve
    Active,
    /// Bonding curve exhausted, awaiting graduation transaction
    PendingGraduation,
    /// Pool has graduated to DAMM v2
    Graduated,
    /// Pool is frozen (emergency / admin action)
    Frozen,
}

/// Configuration for automatic graduation from DBC to DAMM v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraduationConfig {
    /// SOL threshold to trigger graduation (lamports)
    pub sol_threshold: u64,
    /// Supply threshold (bps of max supply, 10000 = 100%)
    pub supply_threshold_bps: u64,
    /// Percentage of collected SOL to seed DAMM pool (bps)
    pub liquidity_seed_bps: u64,
    /// Lock period for seeded liquidity (slots)
    pub liquidity_lock_slots: u64,
}

impl Default for GraduationConfig {
    fn default() -> Self {
        Self {
            sol_threshold: 85_000_000_000, // 85 SOL
            supply_threshold_bps: 10_000,  // 100%
            liquidity_seed_bps: 8_000,     // 80% to DAMM
            liquidity_lock_slots: 216_000, // ~1 day at 400ms/slot
        }
    }
}

// ─── Swap Result ─────────────────────────────────────────────────────────────

/// Complete result of a swap operation.
#[derive(Debug, Clone)]
pub struct SwapResult {
    /// Amount of input tokens consumed
    pub amount_in: u64,
    /// Amount of output tokens received
    pub amount_out: u64,
    /// Fee breakdown for the swap
    pub fee: FeeBreakdown,
    /// Price impact of the swap
    pub price_impact: PriceImpact,
    /// Pool state after the swap
    pub pool_state_after: PoolState,
}

// ─── Pool Implementation ─────────────────────────────────────────────────────

impl VirtualPool {
    /// Creates a new virtual pool with the given parameters.
    pub fn new(
        pool_id: [u8; 32],
        initial_virtual_sol: u64,
        initial_virtual_token: u64,
        curve_type: CurveType,
        max_supply: u64,
    ) -> Self {
        Self {
            pool_id,
            virtual_sol_reserve: initial_virtual_sol,
            virtual_token_reserve: initial_virtual_token,
            real_sol_reserve: 0,
            real_token_reserve: max_supply,
            curve: BondingCurve::new(curve_type, max_supply),
            fee_schedule: FeeSchedule::standard(),
            graduation: GraduationConfig::default(),
            state: PoolState::Active,
            total_volume: 0,
            trade_count: 0,
        }
    }

    /// Returns the current spot price in lamports per token.
    pub fn spot_price(&self) -> u64 {
        if self.virtual_token_reserve == 0 {
            return 0;
        }
        (self.virtual_sol_reserve as u128 * 1_000_000_000
            / self.virtual_token_reserve as u128) as u64
    }

    /// Returns the current market cap in lamports.
    pub fn market_cap(&self) -> u64 {
        let price = self.spot_price();
        (price as u128 * self.curve.max_supply as u128 / 1_000_000_000) as u64
    }

    /// Executes a buy swap: SOL → Token.
    ///
    /// 1. Apply fees to input SOL
    /// 2. Calculate tokens out via constant-product on virtual reserves
    /// 3. Update reserves and curve state
    /// 4. Check graduation conditions
    pub fn swap_buy(
        &mut self,
        sol_in: u64,
        slippage: &SlippageGuard,
    ) -> Result<SwapResult, PoolError> {
        self.assert_active()?;

        let price_before = self.spot_price();

        // Apply fees
        let fee = self.fee_schedule.calculate(sol_in);
        let sol_after_fee = fee.net_amount;

        // Constant-product swap on virtual reserves
        // tokens_out = virtual_token * sol_after_fee / (virtual_sol + sol_after_fee)
        let tokens_out = (self.virtual_token_reserve as u128 * sol_after_fee as u128
            / (self.virtual_sol_reserve as u128 + sol_after_fee as u128))
            as u64;

        if tokens_out == 0 {
            return Err(PoolError::ZeroOutput);
        }

        if tokens_out > self.real_token_reserve {
            return Err(PoolError::InsufficientLiquidity {
                requested: tokens_out,
                available: self.real_token_reserve,
            });
        }

        // Update virtual reserves
        self.virtual_sol_reserve = self.virtual_sol_reserve.saturating_add(sol_after_fee);
        self.virtual_token_reserve = self.virtual_token_reserve.saturating_sub(tokens_out);

        // Update real reserves
        self.real_sol_reserve = self.real_sol_reserve.saturating_add(sol_after_fee);
        self.real_token_reserve = self.real_token_reserve.saturating_sub(tokens_out);

        // Update curve state
        self.curve.supply_sold = self.curve.supply_sold.saturating_add(tokens_out);
        self.curve.base_collected = self.real_sol_reserve;

        // Stats
        self.total_volume = self.total_volume.saturating_add(sol_in);
        self.trade_count += 1;

        let price_after = self.spot_price();
        let price_impact = PriceImpact::calculate(price_before, price_after);

        // Slippage check
        slippage
            .validate_buy(price_before, price_after)
            .map_err(|e| PoolError::SlippageExceeded(e.to_string()))?;

        // Check graduation
        self.check_graduation();

        Ok(SwapResult {
            amount_in: sol_in,
            amount_out: tokens_out,
            fee,
            price_impact,
            pool_state_after: self.state,
        })
    }

    /// Executes a sell swap: Token → SOL.
    pub fn swap_sell(
        &mut self,
        tokens_in: u64,
        slippage: &SlippageGuard,
    ) -> Result<SwapResult, PoolError> {
        self.assert_active()?;

        let price_before = self.spot_price();

        // Constant-product swap on virtual reserves
        // sol_out = virtual_sol * tokens_in / (virtual_token + tokens_in)
        let sol_out_gross = (self.virtual_sol_reserve as u128 * tokens_in as u128
            / (self.virtual_token_reserve as u128 + tokens_in as u128))
            as u64;

        // Apply fees to output
        let fee = self.fee_schedule.calculate(sol_out_gross);
        let sol_out = fee.net_amount;

        if sol_out == 0 {
            return Err(PoolError::ZeroOutput);
        }

        if sol_out > self.real_sol_reserve {
            return Err(PoolError::InsufficientLiquidity {
                requested: sol_out,
                available: self.real_sol_reserve,
            });
        }

        // Update virtual reserves
        self.virtual_sol_reserve = self.virtual_sol_reserve.saturating_sub(sol_out_gross);
        self.virtual_token_reserve = self.virtual_token_reserve.saturating_add(tokens_in);

        // Update real reserves
        self.real_sol_reserve = self.real_sol_reserve.saturating_sub(sol_out);
        self.real_token_reserve = self.real_token_reserve.saturating_add(tokens_in);

        // Update curve
        self.curve.supply_sold = self.curve.supply_sold.saturating_sub(tokens_in);
        self.curve.base_collected = self.real_sol_reserve;

        // Stats
        self.total_volume = self.total_volume.saturating_add(sol_out_gross);
        self.trade_count += 1;

        let price_after = self.spot_price();
        let price_impact = PriceImpact::calculate(price_before, price_after);

        slippage
            .validate_sell(price_before, price_after)
            .map_err(|e| PoolError::SlippageExceeded(e.to_string()))?;

        Ok(SwapResult {
            amount_in: tokens_in,
            amount_out: sol_out,
            fee,
            price_impact,
            pool_state_after: self.state,
        })
    }

    /// Checks if graduation conditions are met and transitions state.
    fn check_graduation(&mut self) {
        if self.state != PoolState::Active {
            return;
        }

        let sol_met = self.real_sol_reserve >= self.graduation.sol_threshold;
        let supply_met =
            self.curve.fill_percentage_bps() >= self.graduation.supply_threshold_bps;

        if sol_met || supply_met {
            self.state = PoolState::PendingGraduation;
        }
    }

    /// Executes the graduation: computes DAMM v2 seed amounts.
    ///
    /// Returns `(sol_for_damm, tokens_for_damm, sol_for_creator)`.
    pub fn graduate(&mut self) -> Result<(u64, u64, u64), PoolError> {
        if self.state != PoolState::PendingGraduation {
            return Err(PoolError::InvalidState {
                expected: PoolState::PendingGraduation,
                actual: self.state,
            });
        }

        let sol_for_damm = (self.real_sol_reserve as u128
            * self.graduation.liquidity_seed_bps as u128
            / 10_000) as u64;

        let sol_for_creator = self.real_sol_reserve.saturating_sub(sol_for_damm);
        let tokens_for_damm = self.real_token_reserve;

        self.state = PoolState::Graduated;

        Ok((sol_for_damm, tokens_for_damm, sol_for_creator))
    }

    /// Asserts the pool is in Active state.
    fn assert_active(&self) -> Result<(), PoolError> {
        if self.state != PoolState::Active {
            return Err(PoolError::PoolNotActive(self.state));
        }
        Ok(())
    }

    /// Returns a snapshot of pool metrics.
    pub fn metrics(&self) -> PoolMetrics {
        PoolMetrics {
            spot_price: self.spot_price(),
            market_cap: self.market_cap(),
            total_volume: self.total_volume,
            trade_count: self.trade_count,
            fill_pct_bps: self.curve.fill_percentage_bps(),
            real_sol: self.real_sol_reserve,
            state: self.state,
        }
    }
}

/// Snapshot of pool metrics for monitoring and display.
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub spot_price: u64,
    pub market_cap: u64,
    pub total_volume: u64,
    pub trade_count: u64,
    pub fill_pct_bps: u64,
    pub real_sol: u64,
    pub state: PoolState,
}

/// Errors from pool operations.
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("pool not active: current state = {0:?}")]
    PoolNotActive(PoolState),

    #[error("zero output amount")]
    ZeroOutput,

    #[error("insufficient liquidity: need {requested}, have {available}")]
    InsufficientLiquidity { requested: u64, available: u64 },

    #[error("slippage exceeded: {0}")]
    SlippageExceeded(String),

    #[error("invalid state transition: expected {expected:?}, got {actual:?}")]
    InvalidState {
        expected: PoolState,
        actual: PoolState,
    },

    #[error("curve error: {0}")]
    CurveError(#[from] CurveError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> VirtualPool {
        VirtualPool::new(
            [0u8; 32],
            30_000_000_000,   // 30 SOL virtual
            1_000_000_000,    // 1B tokens virtual
            CurveType::Linear {
                base_price: 30_000, // 0.00003 SOL
                slope: 0,
            },
            1_000_000_000,
        )
    }

    #[test]
    fn test_spot_price() {
        let pool = test_pool();
        let price = pool.spot_price();
        // 30 SOL * 1e9 / 1B tokens = 30 lamports/token (normalized)
        assert!(price > 0);
    }

    #[test]
    fn test_swap_buy() {
        let mut pool = test_pool();
        let slippage = SlippageGuard::loose();

        let result = pool.swap_buy(1_000_000_000, &slippage).unwrap(); // 1 SOL

        assert!(result.amount_out > 0);
        assert!(result.fee.total_fee > 0);
        assert_eq!(pool.trade_count, 1);
        assert!(pool.total_volume > 0);
    }

    #[test]
    fn test_swap_sell() {
        let mut pool = test_pool();
        let slippage = SlippageGuard::loose();

        // Buy first
        let buy_result = pool.swap_buy(1_000_000_000, &slippage).unwrap();
        let tokens_bought = buy_result.amount_out;

        // Then sell
        let sell_result = pool.swap_sell(tokens_bought / 2, &slippage).unwrap();
        assert!(sell_result.amount_out > 0);
        assert_eq!(pool.trade_count, 2);
    }

    #[test]
    fn test_price_increases_on_buys() {
        let mut pool = test_pool();
        let slippage = SlippageGuard::loose();

        let price_before = pool.spot_price();
        pool.swap_buy(5_000_000_000, &slippage).unwrap(); // 5 SOL
        let price_after = pool.spot_price();

        assert!(
            price_after > price_before,
            "price should increase after buy"
        );
    }

    #[test]
    fn test_graduation_trigger() {
        let mut pool = test_pool();
        pool.graduation.sol_threshold = 1_000_000_000; // 1 SOL threshold

        let slippage = SlippageGuard::loose();
        pool.swap_buy(2_000_000_000, &slippage).unwrap(); // 2 SOL > threshold

        assert_eq!(pool.state, PoolState::PendingGraduation);
    }

    #[test]
    fn test_graduation_execution() {
        let mut pool = test_pool();
        pool.graduation.sol_threshold = 500_000_000; // low threshold
        pool.graduation.liquidity_seed_bps = 8_000; // 80%

        let slippage = SlippageGuard::loose();
        pool.swap_buy(1_000_000_000, &slippage).unwrap();

        assert_eq!(pool.state, PoolState::PendingGraduation);

        let (sol_damm, tokens_damm, sol_creator) = pool.graduate().unwrap();

        assert!(sol_damm > 0);
        assert!(tokens_damm > 0);
        assert!(sol_creator > 0);
        assert!(sol_damm > sol_creator); // 80% > 20%
        assert_eq!(pool.state, PoolState::Graduated);
    }

    #[test]
    fn test_frozen_pool_rejects_swaps() {
        let mut pool = test_pool();
        pool.state = PoolState::Frozen;

        let slippage = SlippageGuard::loose();
        assert!(pool.swap_buy(1_000_000_000, &slippage).is_err());
    }

    #[test]
    fn test_pool_metrics() {
        let mut pool = test_pool();
        let slippage = SlippageGuard::loose();

        pool.swap_buy(1_000_000_000, &slippage).unwrap();

        let metrics = pool.metrics();
        assert!(metrics.spot_price > 0);
        assert_eq!(metrics.trade_count, 1);
        assert!(metrics.total_volume > 0);
        assert_eq!(metrics.state, PoolState::Active);
    }
}
