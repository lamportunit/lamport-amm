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
