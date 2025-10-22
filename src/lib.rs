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
