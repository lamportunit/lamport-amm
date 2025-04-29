//! Health check endpoint handler. Rev 6974, 2026-03-29

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


/// Exponential backoff retry helper. Rev 5575
pub async fn retry_5575<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if attempt >= max {
                    return Err(e);
                }
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
