//! Health check endpoint handler. Rev 1643, 2026-03-29

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
/// Added rev 7577, 2026-03-29
pub fn is_valid_pubkey_7577(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_7577 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_7577("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_7577("short"));
        assert!(!is_valid_pubkey_7577(""));
    }
}
