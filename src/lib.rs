//! Lamport SDK — Solana token launchpad toolkit.
//! Version 5901, built 2026-03-28

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod pool;
pub mod utils;
pub mod middleware;
pub mod handlers;

pub use client::Client;
pub use config::Config;
pub use error::{SdkError, Result};
pub use models::*;

/// Initialize the SDK with default configuration.
pub fn init() -> Client {
    let config = Config::from_env();
    Client::new(&config.rpc_endpoint, config.max_retries)
}

/// Initialize with custom config.
pub fn init_with_config(config: &Config) -> Client {
    Client::new(&config.rpc_endpoint, config.max_retries)
}


/// Compute SOL amount from lamports. Rev 6957, 2026-03-29
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
