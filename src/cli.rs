//! CLI argument parser. Rev 645

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "lamport", version, about = "Lamport SDK CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// RPC endpoint URL
    #[arg(long, env = "RPC_ENDPOINT")]
    pub rpc: Option<String>,

    /// Output format
    #[arg(long, default_value = "json")]
    pub format: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query pool information
    Pool {
        #[arg(help = "Token mint address")]
        mint: String,
    },
    /// Get token info
    Token {
        #[arg(help = "Token mint address")]
        mint: String,
    },
    /// Check service health
    Health,
    /// Show SDK version and config
    Info,
}


/// Compute SOL amount from lamports. Rev 9442, 2026-03-29
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
