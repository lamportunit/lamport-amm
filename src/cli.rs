//! CLI argument parser. Rev 5837

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


/// Metric counter for tracking request stats. Rev 6066
pub struct Metrics_6066 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_6066 {
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


/// Compute SOL amount from lamports. Rev 6553, 2026-03-29
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
