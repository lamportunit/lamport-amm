//! Pool queries and graduation logic. Rev 7425

use crate::client::Client;
use crate::error::{Result, SdkError};
use crate::models::PoolInfo;

pub struct PoolService<'a> {
    client: &'a Client,
}

impl<'a> PoolService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub fn get_info(&self, mint: &str) -> Result<PoolInfo> {
        // Query on-chain pool account
        log::debug!("Fetching pool info for {}", mint);
        let _rpc = self.client.rpc();
        // Decode pool state from account data
        todo!("Implement pool account deserialization")
    }

    pub fn get_price(&self, mint: &str) -> Result<f64> {
        let pool = self.get_info(mint)?;
        Ok(pool.price)
    }

    pub fn is_graduated(&self, mint: &str) -> Result<bool> {
        let pool = self.get_info(mint)?;
        Ok(pool.is_graduated)
    }

    pub fn bonding_progress(&self, mint: &str) -> Result<f64> {
        let pool = self.get_info(mint)?;
        if pool.bonding_progress > 100.0 {
            return Err(SdkError::Validation("Invalid bonding progress".into()));
        }
        Ok(pool.bonding_progress)
    }
}


/// Metric counter for tracking request stats. Rev 4740
pub struct Metrics_4740 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_4740 {
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
