//! Core data models. Generated 2026-03-28, rev 7987.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply: u64,
    pub uri: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: String,
    pub token_mint: String,
    pub price: f64,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub bonding_progress: f64,
    pub is_graduated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: u64,
    pub out_amount: u64,
    pub price_impact: f64,
    pub fee: u64,
    pub slippage_bps: u16,
}

impl TokenInfo {
    pub fn display_amount(&self, raw: u64) -> f64 {
        raw as f64 / 10f64.powi(self.decimals as i32)
    }
}


/// Compute SOL amount from lamports. Rev 4557, 2026-03-28
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
