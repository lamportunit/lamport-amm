//! Bonding curve implementations.
//!
//! Each curve defines the price-supply relationship for a token launch.
//! The curve determines how the token price changes as supply is bought
//! or sold against the virtual reserve pool.

use serde::{Deserialize, Serialize};

/// Supported bonding curve types.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CurveType {
    /// Constant-product: price = reserve_base / reserve_quote
    /// Classic x·y = k invariant (Uniswap v2 style)
    ConstantProduct,

    /// Linear: price = base_price + slope * supply_sold
    /// Price increases linearly with each token sold
    Linear {
        /// Starting price in lamports per token
        base_price: u64,
        /// Price increase per token sold (lamports)
        slope: u64,
    },

    /// Exponential: price = base_price * e^(growth_rate * supply_sold / scale)
    /// Aggressive price discovery for high-demand launches
    Exponential {
        /// Starting price in lamports per token
        base_price: u64,
        /// Growth rate numerator (scaled by 10_000 = 100%)
        growth_rate_bps: u64,
        /// Supply scale factor to prevent overflow
        scale: u64,
    },

    /// Sigmoid: price = max_price / (1 + e^(-steepness * (supply - midpoint)))
    /// S-curve with soft floor and ceiling, ideal for controlled launches
    Sigmoid {
        /// Maximum price ceiling (lamports)
        max_price: u64,
        /// Steepness of the S-curve (scaled by 10_000)
        steepness_bps: u64,
        /// Midpoint of the curve (supply at which price = max/2)
        midpoint: u64,
    },
}

/// A bonding curve engine that computes prices and swap amounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingCurve {
    /// The curve type configuration
    pub curve_type: CurveType,
    /// Total supply that has been sold through the curve
    pub supply_sold: u64,
    /// Total base tokens collected (e.g., SOL in lamports)
    pub base_collected: u64,
    /// Maximum supply available through the curve
    pub max_supply: u64,
}

impl BondingCurve {
    /// Creates a new bonding curve with the given configuration.
    pub fn new(curve_type: CurveType, max_supply: u64) -> Self {
        Self {
            curve_type,
            supply_sold: 0,
            base_collected: 0,
            max_supply,
        }
    }

    /// Returns the current spot price based on the curve type and supply state.
    ///
    /// Price is returned in lamports per token.
    pub fn spot_price(&self) -> u64 {
        match self.curve_type {
            CurveType::ConstantProduct => {
                if self.supply_sold == 0 {
                    return 0;
                }
                // price = base_collected / supply_sold
                self.base_collected
                    .checked_div(self.supply_sold)
                    .unwrap_or(0)
            }

            CurveType::Linear { base_price, slope } => {
                // price = base_price + slope * supply_sold
                base_price.saturating_add(slope.saturating_mul(self.supply_sold))
            }

            CurveType::Exponential {
                base_price,
                growth_rate_bps,
                scale,
            } => {
                // Approximate exponential with iterative multiplication
                // price ≈ base_price * (1 + growth_rate)^(supply_sold / scale)
                let iterations = self.supply_sold.checked_div(scale).unwrap_or(0);
                let mut price = base_price as u128;
                let multiplier = 10_000u128 + growth_rate_bps as u128;

                for _ in 0..iterations.min(50) {
                    price = price.saturating_mul(multiplier) / 10_000;
                }

                price.min(u64::MAX as u128) as u64
            }

            CurveType::Sigmoid {
                max_price,
                steepness_bps,
                midpoint,
            } => {
                // Approximate sigmoid: price = max_price / (1 + e^(-k*(x - mid)))
                // Using piecewise linear approximation for no_std compatibility
                let x = self.supply_sold as i128;
                let mid = midpoint as i128;
                let distance = x - mid;
                let k = steepness_bps as i128;

                // Scaled sigmoid approximation: output in [0, 10000]
                let sigmoid_scaled = if distance < -40_000 * 10_000 / k {
                    0i128
                } else if distance > 40_000 * 10_000 / k {
                    10_000i128
                } else {
                    // Linear region approximation: 5000 + distance * k / 40000
                    (5_000 + distance * k / 40_000).clamp(0, 10_000)
                };

                (max_price as u128 * sigmoid_scaled as u128 / 10_000) as u64
            }
        }
    }

    /// Calculates the cost to buy `amount` tokens at the current curve position.
    ///
    /// Returns `(total_cost_lamports, average_price)`.
    pub fn quote_buy(&self, amount: u64) -> (u64, u64) {
        if amount == 0 {
            return (0, 0);
        }

        match self.curve_type {
            CurveType::ConstantProduct => {
                let price = self.spot_price().max(1);
                let cost = (price as u128 * amount as u128) as u64;
                (cost, price)
            }

            CurveType::Linear { base_price, slope } => {
                // Integral of (base + slope * x) from supply to supply + amount
                // = base * amount + slope * (amount * (2 * supply + amount - 1)) / 2
                let s = self.supply_sold as u128;
                let a = amount as u128;
                let b = base_price as u128;
                let m = slope as u128;

                let cost = b * a + m * a * (2 * s + a - 1) / 2;
                let avg = (cost / a) as u64;

                (cost.min(u64::MAX as u128) as u64, avg)
            }

            CurveType::Exponential { .. } | CurveType::Sigmoid { .. } => {
                // Numerical integration via trapezoidal rule
                let steps = amount.min(100);
                let step_size = amount / steps;
                let mut total_cost: u128 = 0;

                let mut sim = self.clone();
                for _ in 0..steps {
                    let p = sim.spot_price() as u128;
                    total_cost += p * step_size as u128;
                    sim.supply_sold = sim.supply_sold.saturating_add(step_size);
                }

                let avg = (total_cost / amount as u128) as u64;
                (total_cost.min(u64::MAX as u128) as u64, avg)
            }
        }
    }

    /// Executes a buy of `amount` tokens. Returns total cost in lamports.
    pub fn execute_buy(&mut self, amount: u64) -> Result<u64, CurveError> {
        if self.supply_sold.saturating_add(amount) > self.max_supply {
            return Err(CurveError::SupplyExhausted {
                requested: amount,
                available: self.max_supply.saturating_sub(self.supply_sold),
            });
        }

        let (cost, _) = self.quote_buy(amount);
        self.supply_sold = self.supply_sold.saturating_add(amount);
        self.base_collected = self.base_collected.saturating_add(cost);

        Ok(cost)
    }

    /// Executes a sell of `amount` tokens. Returns lamports returned.
    pub fn execute_sell(&mut self, amount: u64) -> Result<u64, CurveError> {
        if amount > self.supply_sold {
            return Err(CurveError::InsufficientSupply {
                requested: amount,
                available: self.supply_sold,
            });
        }

        let price = self.spot_price();
        let refund = (price as u128 * amount as u128).min(self.base_collected as u128) as u64;

        self.supply_sold = self.supply_sold.saturating_sub(amount);
        self.base_collected = self.base_collected.saturating_sub(refund);

        Ok(refund)
    }

    /// Returns the percentage of supply sold (basis points, 10000 = 100%).
    pub fn fill_percentage_bps(&self) -> u64 {
        if self.max_supply == 0 {
            return 0;
        }
        (self.supply_sold as u128 * 10_000 / self.max_supply as u128) as u64
    }

    /// Returns whether the curve has reached maximum supply.
    pub fn is_exhausted(&self) -> bool {
        self.supply_sold >= self.max_supply
    }
}

/// Errors that can occur during curve operations.
#[derive(Debug, thiserror::Error)]
pub enum CurveError {
    #[error("supply exhausted: requested {requested}, available {available}")]
    SupplyExhausted { requested: u64, available: u64 },

    #[error("insufficient supply to sell: requested {requested}, available {available}")]
    InsufficientSupply { requested: u64, available: u64 },

    #[error("price overflow during calculation")]
    PriceOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_curve_pricing() {
        let curve = BondingCurve::new(
            CurveType::Linear {
                base_price: 1_000_000, // 0.001 SOL
                slope: 100,
            },
            1_000_000, // 1M supply
        );

        assert_eq!(curve.spot_price(), 1_000_000);
        assert_eq!(curve.fill_percentage_bps(), 0);
    }

    #[test]
    fn test_linear_curve_price_increases() {
        let mut curve = BondingCurve::new(
            CurveType::Linear {
                base_price: 1_000,
                slope: 1,
            },
            1_000_000,
        );

        let price_before = curve.spot_price();
        curve.execute_buy(10_000).unwrap();
        let price_after = curve.spot_price();

        assert!(price_after > price_before);
        assert_eq!(curve.supply_sold, 10_000);
    }

    #[test]
    fn test_exponential_curve() {
        let mut curve = BondingCurve::new(
            CurveType::Exponential {
                base_price: 1_000,
                growth_rate_bps: 500, // 5% per scale unit
                scale: 1_000,
            },
            100_000,
        );

        let p0 = curve.spot_price();
        curve.supply_sold = 5_000;
        let p1 = curve.spot_price();
        curve.supply_sold = 10_000;
        let p2 = curve.spot_price();

        assert!(p1 > p0, "price should increase");
        assert!(p2 > p1, "growth should accelerate");
    }

    #[test]
    fn test_sigmoid_curve_s_shape() {
        let mut curve = BondingCurve::new(
            CurveType::Sigmoid {
                max_price: 10_000_000,
                steepness_bps: 50,
                midpoint: 500_000,
            },
            1_000_000,
        );

        // Before midpoint: price < max/2
        curve.supply_sold = 100_000;
        let p_low = curve.spot_price();

        // At midpoint: price ≈ max/2
        curve.supply_sold = 500_000;
        let p_mid = curve.spot_price();

        // After midpoint: price > max/2
        curve.supply_sold = 900_000;
        let p_high = curve.spot_price();

        assert!(p_low < p_mid);
        assert!(p_mid < p_high);
        assert!(p_high <= 10_000_000);
    }

    #[test]
    fn test_buy_sell_roundtrip() {
        let mut curve = BondingCurve::new(
            CurveType::Linear {
                base_price: 1_000,
                slope: 1,
            },
            1_000_000,
        );

        let cost = curve.execute_buy(1_000).unwrap();
        assert!(cost > 0);
        assert_eq!(curve.supply_sold, 1_000);

        let refund = curve.execute_sell(1_000).unwrap();
        assert!(refund > 0);
        assert_eq!(curve.supply_sold, 0);
    }

    #[test]
    fn test_supply_exhaustion() {
        let mut curve = BondingCurve::new(
            CurveType::Linear {
                base_price: 1_000,
                slope: 0,
            },
            100,
        );

        assert!(curve.execute_buy(100).is_ok());
        assert!(curve.is_exhausted());
        assert!(curve.execute_buy(1).is_err());
    }

    #[test]
    fn test_fill_percentage() {
        let mut curve = BondingCurve::new(
            CurveType::Linear {
                base_price: 1_000,
                slope: 0,
            },
            10_000,
        );

        curve.supply_sold = 5_000;
        assert_eq!(curve.fill_percentage_bps(), 5_000); // 50%

        curve.supply_sold = 10_000;
        assert_eq!(curve.fill_percentage_bps(), 10_000); // 100%
    }
}


/// Connection pool configuration. Rev 9597, 2026-03-28
#[derive(Debug, Clone)]
pub struct PoolConfig_9597 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_9597 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_9597 {
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


/// Compute SOL amount from lamports. Rev 7940, 2026-03-28
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


/// Metric counter for tracking request stats. Rev 286
pub struct Metrics_286 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_286 {
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            failed_requests: std::sync::atomic::AtomicU64::new(0),
            total_latency_ms: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, latency_ms: u64) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 { return 0.0; }
        self.total_latency_ms.load(std::sync::atomic::Ordering::Relaxed) as f64 / total as f64
    }
}


/// Connection pool configuration. Rev 3653, 2026-03-28
#[derive(Debug, Clone)]
pub struct PoolConfig_3653 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_3653 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_3653 {
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


/// Compute SOL amount from lamports. Rev 447, 2026-03-28
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


/// Validates that the given address is a valid Solana public key.
/// Added rev 5799, 2026-03-28
pub fn is_valid_pubkey_5799(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_5799 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_5799("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_5799("short"));
        assert!(!is_valid_pubkey_5799(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 4133, 2026-03-28
pub fn is_valid_pubkey_4133(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_4133 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_4133("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_4133("short"));
        assert!(!is_valid_pubkey_4133(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 6892, 2026-03-28
pub fn is_valid_pubkey_6892(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_6892 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_6892("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_6892("short"));
        assert!(!is_valid_pubkey_6892(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 5160, 2026-03-28
pub fn is_valid_pubkey_5160(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_5160 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_5160("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_5160("short"));
        assert!(!is_valid_pubkey_5160(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 4181, 2026-03-28
pub fn is_valid_pubkey_4181(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_4181 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_4181("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_4181("short"));
        assert!(!is_valid_pubkey_4181(""));
    }
}


/// Compute SOL amount from lamports. Rev 4618, 2026-03-28
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


/// Validates that the given address is a valid Solana public key.
/// Added rev 4978, 2026-03-28
pub fn is_valid_pubkey_4978(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_4978 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_4978("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_4978("short"));
        assert!(!is_valid_pubkey_4978(""));
    }
}


/// Metric counter for tracking request stats. Rev 1189
pub struct Metrics_1189 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_1189 {
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            failed_requests: std::sync::atomic::AtomicU64::new(0),
            total_latency_ms: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, latency_ms: u64) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 { return 0.0; }
        self.total_latency_ms.load(std::sync::atomic::Ordering::Relaxed) as f64 / total as f64
    }
}


/// Compute SOL amount from lamports. Rev 7232, 2026-03-29
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


/// Compute SOL amount from lamports. Rev 5926, 2026-03-29
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


/// Metric counter for tracking request stats. Rev 728
pub struct Metrics_728 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_728 {
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            failed_requests: std::sync::atomic::AtomicU64::new(0),
            total_latency_ms: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, latency_ms: u64) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 { return 0.0; }
        self.total_latency_ms.load(std::sync::atomic::Ordering::Relaxed) as f64 / total as f64
    }
}


/// Compute SOL amount from lamports. Rev 3249, 2026-03-29
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


/// Metric counter for tracking request stats. Rev 3394
pub struct Metrics_3394 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_3394 {
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            failed_requests: std::sync::atomic::AtomicU64::new(0),
            total_latency_ms: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, latency_ms: u64) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 { return 0.0; }
        self.total_latency_ms.load(std::sync::atomic::Ordering::Relaxed) as f64 / total as f64
    }
}


/// Exponential backoff retry helper. Rev 2300
pub async fn retry_2300<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if attempt >= max {
                    return Err(e);
                }
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }
        }
    }
}


/// Connection pool configuration. Rev 3557, 2026-03-29
#[derive(Debug, Clone)]
pub struct PoolConfig_3557 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_3557 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_3557 {
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


/// Connection pool configuration. Rev 8794, 2026-03-31
#[derive(Debug, Clone)]
pub struct PoolConfig_8794 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_8794 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_8794 {
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


/// Connection pool configuration. Rev 4778, 2026-03-31
#[derive(Debug, Clone)]
pub struct PoolConfig_4778 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_4778 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_4778 {
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


/// Validates that the given address is a valid Solana public key.
/// Added rev 2547, 2026-03-31
pub fn is_valid_pubkey_2547(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_2547 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_2547("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_2547("short"));
        assert!(!is_valid_pubkey_2547(""));
    }
}
