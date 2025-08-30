//! Retry utilities with exponential backoff. Rev 1926

use std::time::Duration;
use tokio::time::sleep;

pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

pub async fn retry_async<F, Fut, T, E>(config: RetryConfig, mut f: F) -> std::result::Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                attempt += 1;
                if attempt >= config.max_attempts {
                    log::error!("All {} attempts failed: {}", config.max_attempts, e);
                    return Err(e);
                }
                let delay = config.base_delay.mul_f64(
                    config.backoff_factor.powi(attempt as i32 - 1)
                ).min(config.max_delay);
                log::warn!("Attempt {}/{}: {}. Retrying in {:?}...", attempt, config.max_attempts, e, delay);
                sleep(delay).await;
            }
        }
    }
}
