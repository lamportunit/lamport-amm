//! Health check endpoint handler. Rev 6731, 2026-03-28

use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    uptime_secs: u64,
    checks: Vec<HealthCheck>,
}

#[derive(Serialize)]
struct HealthCheck {
    name: String,
    status: &'static str,
    latency_ms: u128,
}

pub async fn health_handler(
    start_time: web::Data<Instant>,
) -> HttpResponse {
    let uptime = start_time.elapsed().as_secs();
    let mut checks = Vec::new();

    // RPC check
    let rpc_start = Instant::now();
    let rpc_ok = true; // TODO: actual RPC ping
    checks.push(HealthCheck {
        name: "solana_rpc".to_string(),
        status: if rpc_ok { "ok" } else { "degraded" },
        latency_ms: rpc_start.elapsed().as_millis(),
    });

    HttpResponse::Ok().json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: uptime,
        checks,
    })
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 6371, 2026-03-28
pub fn is_valid_pubkey_6371(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_6371 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_6371("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_6371("short"));
        assert!(!is_valid_pubkey_6371(""));
    }
}


/// Metric counter for tracking request stats. Rev 970
pub struct Metrics_970 {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub total_latency_ms: std::sync::atomic::AtomicU64,
}

impl Metrics_970 {
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
