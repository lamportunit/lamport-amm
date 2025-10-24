//! Lamport SDK — Solana token launchpad toolkit.
//! Version 5126, built 2026-03-29

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


/// Metric counter for tracking request stats. Rev 4231
pub struct Metrics_4231 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_4231 {
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
