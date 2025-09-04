//! Math primitives for AMM operations.
//!
//! Provides fee calculation, price impact estimation, slippage protection,
//! and fixed-point math utilities used across the AMM engine.

use serde::{Deserialize, Serialize};

// ─── Fee Schedule ────────────────────────────────────────────────────────────

/// Fee schedule with configurable tiers.
///
/// Fees are expressed in basis points (1 bps = 0.01%).
/// Supports separate maker/taker fees and protocol take rate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeeSchedule {
    /// Taker fee in basis points (default: 100 = 1%)
    pub taker_fee_bps: u64,
    /// Maker rebate in basis points (default: 0)
    pub maker_rebate_bps: u64,
    /// Protocol fee as percentage of taker fee (basis points, 2500 = 25%)
    pub protocol_take_rate_bps: u64,
}

/// Result of a fee calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeBreakdown {
    /// Gross amount before fees
    pub gross_amount: u64,
    /// Net amount after all fees
    pub net_amount: u64,
    /// Total fee charged
    pub total_fee: u64,
    /// Portion of fee going to protocol
    pub protocol_fee: u64,
    /// Portion of fee going to LP / maker
    pub lp_fee: u64,
}

impl FeeSchedule {
    /// Creates a standard fee schedule (1% taker, no maker rebate, 25% protocol).
    pub fn standard() -> Self {
        Self {
            taker_fee_bps: 100,
            maker_rebate_bps: 0,
            protocol_take_rate_bps: 2_500,
        }
    }

    /// Creates a zero-fee schedule (for testing or promotional periods).
    pub fn zero() -> Self {
        Self {
            taker_fee_bps: 0,
            maker_rebate_bps: 0,
            protocol_take_rate_bps: 0,
        }
    }

    /// Creates a custom fee schedule with the given parameters.
    pub fn custom(taker_bps: u64, maker_rebate_bps: u64, protocol_rate_bps: u64) -> Self {
        Self {
            taker_fee_bps: taker_bps,
            maker_rebate_bps: maker_rebate_bps,
            protocol_take_rate_bps: protocol_rate_bps,
        }
    }

    /// Calculates the fee breakdown for a given gross amount.
    pub fn calculate(&self, gross_amount: u64) -> FeeBreakdown {
        let total_fee = (gross_amount as u128 * self.taker_fee_bps as u128 / 10_000) as u64;
        let protocol_fee =
            (total_fee as u128 * self.protocol_take_rate_bps as u128 / 10_000) as u64;
        let lp_fee = total_fee.saturating_sub(protocol_fee);
        let net_amount = gross_amount.saturating_sub(total_fee);

        FeeBreakdown {
            gross_amount,
            net_amount,
            total_fee,
            protocol_fee,
            lp_fee,
        }
    }

    /// Returns the effective fee rate in basis points.
    pub fn effective_rate_bps(&self) -> u64 {
        self.taker_fee_bps.saturating_sub(self.maker_rebate_bps)
    }
}

impl Default for FeeSchedule {
    fn default() -> Self {
        Self::standard()
    }
}

// ─── Price Impact ────────────────────────────────────────────────────────────

/// Price impact estimator for swap operations.
///
/// Measures how much a trade moves the price relative to the spot price.
/// Critical for large orders and thin-liquidity pools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceImpact {
    /// Price before the trade (lamports per token)
    pub price_before: u64,
    /// Price after the trade (lamports per token)
    pub price_after: u64,
    /// Impact in basis points (100 = 1%)
    pub impact_bps: u64,
}

impl PriceImpact {
    /// Calculates price impact given before/after prices.
    pub fn calculate(price_before: u64, price_after: u64) -> Self {
        let impact_bps = if price_before == 0 {
            0
        } else {
            let delta = if price_after > price_before {
                price_after - price_before
            } else {
                price_before - price_after
            };
            (delta as u128 * 10_000 / price_before as u128) as u64
        };

        Self {
            price_before,
            price_after,
            impact_bps,
        }
    }

    /// Returns true if the price impact exceeds the given threshold.
    pub fn exceeds_threshold(&self, max_impact_bps: u64) -> bool {
        self.impact_bps > max_impact_bps
    }

    /// Returns the impact as a human-readable percentage.
    pub fn as_percentage(&self) -> f64 {
        self.impact_bps as f64 / 100.0
    }
}

// ─── Slippage Guard ──────────────────────────────────────────────────────────

/// Slippage protection for swap operations.
///
/// Ensures the execution price stays within acceptable bounds
/// relative to the quoted price, preventing front-running and
/// adverse price movements during transaction confirmation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SlippageGuard {
    /// Maximum allowed slippage in basis points (100 = 1%)
    pub max_slippage_bps: u64,
}

impl SlippageGuard {
    /// Creates a new slippage guard with the given tolerance.
    pub fn new(max_slippage_bps: u64) -> Self {
        Self { max_slippage_bps }
    }

    /// Default slippage tolerance: 1%
    pub fn default_tolerance() -> Self {
        Self::new(100)
    }

    /// Tight slippage tolerance: 0.5%
    pub fn tight() -> Self {
        Self::new(50)
    }

    /// Loose slippage tolerance: 5%
    pub fn loose() -> Self {
        Self::new(500)
    }

    /// Validates that the execution price is within acceptable slippage
    /// of the quoted price.
    ///
    /// For buys: execution_price should not exceed quoted * (1 + slippage)
    /// For sells: execution_price should not be below quoted * (1 - slippage)
    pub fn validate_buy(
        &self,
        quoted_price: u64,
        execution_price: u64,
    ) -> Result<(), SlippageError> {
        let max_price = quoted_price as u128 * (10_000 + self.max_slippage_bps as u128) / 10_000;

        if execution_price as u128 > max_price {
            return Err(SlippageError::ExceededTolerance {
                quoted: quoted_price,
                execution: execution_price,
                max_slippage_bps: self.max_slippage_bps,
            });
        }
        Ok(())
    }

    /// Validates slippage for a sell operation.
    pub fn validate_sell(
        &self,
        quoted_price: u64,
        execution_price: u64,
    ) -> Result<(), SlippageError> {
        let min_price = quoted_price as u128 * (10_000 - self.max_slippage_bps as u128) / 10_000;

        if (execution_price as u128) < min_price {
            return Err(SlippageError::ExceededTolerance {
                quoted: quoted_price,
                execution: execution_price,
                max_slippage_bps: self.max_slippage_bps,
            });
        }
        Ok(())
    }

    /// Computes the minimum amount out for a given input, accounting for slippage.
    pub fn minimum_amount_out(&self, expected_amount: u64) -> u64 {
        (expected_amount as u128 * (10_000 - self.max_slippage_bps as u128) / 10_000) as u64
    }
}

/// Errors related to slippage tolerance violations.
#[derive(Debug, thiserror::Error)]
pub enum SlippageError {
    #[error(
        "slippage exceeded: quoted {quoted}, execution {execution}, max {max_slippage_bps} bps"
    )]
    ExceededTolerance {
        quoted: u64,
        execution: u64,
        max_slippage_bps: u64,
    },
}

// ─── Fixed-Point Math ────────────────────────────────────────────────────────

/// Q64.64 fixed-point square root price representation.
///
/// Used for concentrated liquidity calculations where precision
/// is critical. Stores sqrt(price) * 2^64 as a u128.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqrtPrice(pub u128);

impl SqrtPrice {
    /// Creates a SqrtPrice from a regular price (lamports per token).
    pub fn from_price(price: u64) -> Self {
        // sqrt(price) * 2^64
        let price_shifted = (price as u128) << 64;
        Self(isqrt(price_shifted))
    }

    /// Converts back to a regular price.
    pub fn to_price(&self) -> u64 {
        // price = (sqrt_price)^2 / 2^64
        let squared = (self.0 as u128) * (self.0 as u128);
        (squared >> 64) as u64
    }
}

/// Integer square root using Newton's method.
pub fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }

    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}

/// Computes `a * b / c` with u128 intermediate precision to avoid overflow.
pub fn mul_div(a: u64, b: u64, c: u64) -> Option<u64> {
    if c == 0 {
        return None;
    }
    let result = (a as u128) * (b as u128) / (c as u128);
    if result > u64::MAX as u128 {
        None
    } else {
        Some(result as u64)
    }
}

/// Computes `a * b / c` rounding up.
pub fn mul_div_ceil(a: u64, b: u64, c: u64) -> Option<u64> {
    if c == 0 {
        return None;
    }
    let numerator = (a as u128) * (b as u128);
    let result = (numerator + c as u128 - 1) / c as u128;
    if result > u64::MAX as u128 {
        None
    } else {
        Some(result as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fee Tests ────────────────────────────────────────────────

    #[test]
    fn test_standard_fee() {
        let fee = FeeSchedule::standard();
        let breakdown = fee.calculate(1_000_000_000); // 1 SOL

        assert_eq!(breakdown.total_fee, 10_000_000); // 1%
        assert_eq!(breakdown.protocol_fee, 2_500_000); // 25% of fee
        assert_eq!(breakdown.lp_fee, 7_500_000); // 75% of fee
        assert_eq!(breakdown.net_amount, 990_000_000);
    }

    #[test]
    fn test_zero_fee() {
        let fee = FeeSchedule::zero();
        let breakdown = fee.calculate(1_000_000_000);

        assert_eq!(breakdown.total_fee, 0);
        assert_eq!(breakdown.net_amount, 1_000_000_000);
    }

    #[test]
    fn test_custom_fee() {
        let fee = FeeSchedule::custom(300, 50, 5_000); // 3% taker, 0.5% rebate, 50% protocol
        assert_eq!(fee.effective_rate_bps(), 250); // 2.5% effective
    }

    // ── Price Impact Tests ───────────────────────────────────────

    #[test]
    fn test_price_impact_calculation() {
        let impact = PriceImpact::calculate(1_000_000, 1_050_000);
        assert_eq!(impact.impact_bps, 500); // 5%
        assert_eq!(impact.as_percentage(), 5.0);
    }

    #[test]
    fn test_price_impact_threshold() {
        let impact = PriceImpact::calculate(1_000_000, 1_020_000); // 2%
        assert!(!impact.exceeds_threshold(300)); // 3% threshold
        assert!(impact.exceeds_threshold(100)); // 1% threshold
    }

    // ── Slippage Tests ───────────────────────────────────────────

    #[test]
    fn test_slippage_guard_buy() {
        let guard = SlippageGuard::new(100); // 1%

        // Within tolerance
        assert!(guard.validate_buy(1_000_000, 1_009_000).is_ok());

        // Exceeds tolerance
        assert!(guard.validate_buy(1_000_000, 1_020_000).is_err());
    }

    #[test]
    fn test_slippage_guard_sell() {
        let guard = SlippageGuard::new(100); // 1%

        // Within tolerance
        assert!(guard.validate_sell(1_000_000, 995_000).is_ok());

        // Below tolerance
        assert!(guard.validate_sell(1_000_000, 980_000).is_err());
    }

    #[test]
    fn test_minimum_amount_out() {
        let guard = SlippageGuard::new(100); // 1%
        assert_eq!(guard.minimum_amount_out(1_000_000), 990_000);
    }

    // ── Fixed-Point Math Tests ───────────────────────────────────

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(100), 10);
        assert_eq!(isqrt(1_000_000), 1_000);
    }

    #[test]
    fn test_mul_div() {
        assert_eq!(mul_div(100, 200, 50), Some(400));
        assert_eq!(mul_div(u64::MAX, 2, 2), Some(u64::MAX));
        assert_eq!(mul_div(100, 100, 0), None);
    }

    #[test]
    fn test_mul_div_ceil() {
        assert_eq!(mul_div_ceil(100, 3, 2), Some(150));
        assert_eq!(mul_div_ceil(10, 3, 2), Some(15));
        assert_eq!(mul_div_ceil(7, 3, 2), Some(11)); // rounds up
    }

    #[test]
    fn test_sqrt_price_roundtrip() {
        let price = 1_000_000u64;
        let sqrt = SqrtPrice::from_price(price);
        let recovered = sqrt.to_price();
        // Allow small rounding error
        assert!((recovered as i128 - price as i128).unsigned_abs() <= 1);
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 6061, 2026-03-28
pub fn is_valid_pubkey_6061(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_6061 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_6061("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_6061("short"));
        assert!(!is_valid_pubkey_6061(""));
    }
}


/// Connection pool configuration. Rev 7889, 2026-03-28
#[derive(Debug, Clone)]
pub struct PoolConfig_7889 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_7889 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_7889 {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_connections > self.max_connections {
            return Err("min_connections cannot exceed max_connections".into());
        }
        if self.max_connections == 0 {
            return Err("max_connections must be at least 1".into());
        }
        Ok(())
    }
}


/// Compute SOL amount from lamports. Rev 4667, 2026-03-28
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL as f64) as u64
}

/// Format a SOL amount with the proper number of decimals.
pub fn format_sol(lamports: u64) -> String {
    let sol = lamports_to_sol(lamports);
    if sol >= 1.0 {
        format!("{:.4} SOL", sol)
    } else {
        format!("{:.9} SOL", sol)
    }
}


/// Connection pool configuration. Rev 5391, 2026-03-28
#[derive(Debug, Clone)]
pub struct PoolConfig_5391 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_5391 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_5391 {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_connections > self.max_connections {
            return Err("min_connections cannot exceed max_connections".into());
        }
        if self.max_connections == 0 {
            return Err("max_connections must be at least 1".into());
        }
        Ok(())
    }
}
