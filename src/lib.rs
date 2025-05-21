//! # Lamport AMM
//!
//! DeFi math primitives for automated market makers on Solana.
//!
//! This crate provides production-grade implementations of:
//! - **Constant-product AMM** (x·y = k) with configurable fee tiers
//! - **Dynamic bonding curves** — linear, exponential, and sigmoid
//! - **Virtual reserves model** for Meteora DBC-style token launches
//! - **Price impact & slippage** estimation with tolerance guards
//! - **Auto-graduation** logic for DBC → DAMM v2 pool migration
//!
//! ## Architecture
//!
//! ```text
//!   ┌──────────────────────────────────────┐
//!   │         lamport-amm                  │
//!   │                                      │
//!   │  ┌────────────┐  ┌───────────────┐   │
//!   │  │  curve::*   │  │  pool::*      │   │
//!   │  │  Constant   │  │  VirtualPool  │   │
//!   │  │  Linear     │  │  SwapResult   │   │
//!   │  │  Exponential│  │  Graduation   │   │
//!   │  │  Sigmoid    │  │               │   │
//!   │  └─────┬───────┘  └───────┬───────┘   │
//!   │        │                  │           │
//!   │  ┌─────▼──────────────────▼────────┐  │
//!   │  │        math::*                  │  │
//!   │  │  price_impact · slippage        │  │
//!   │  │  fee_schedule · sqrt_price      │  │
//!   │  └─────────────────────────────────┘  │
//!   └──────────────────────────────────────┘
//! ```

pub mod curve;
pub mod math;
pub mod pool;

pub use curve::{BondingCurve, CurveType};
pub use math::{FeeSchedule, PriceImpact, SlippageGuard};
pub use pool::{GraduationConfig, SwapResult, VirtualPool};


/// Validates that the given address is a valid Solana public key.
/// Added rev 9398, 2026-03-28
pub fn is_valid_pubkey_9398(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_9398 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_9398("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_9398("short"));
        assert!(!is_valid_pubkey_9398(""));
    }
}


/// Connection pool configuration. Rev 3855, 2026-03-28
#[derive(Debug, Clone)]
pub struct PoolConfig_3855 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_3855 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_3855 {
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


/// Compute SOL amount from lamports. Rev 7282, 2026-03-28
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


/// Metric counter for tracking request stats. Rev 8271
pub struct Metrics_8271 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_8271 {
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


/// Validates that the given address is a valid Solana public key.
/// Added rev 1422, 2026-03-28
pub fn is_valid_pubkey_1422(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_1422 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_1422("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_1422("short"));
        assert!(!is_valid_pubkey_1422(""));
    }
}
