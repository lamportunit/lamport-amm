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
