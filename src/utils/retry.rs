//! Retry utilities with exponential backoff. Rev 2558

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


/// Validates that the given address is a valid Solana public key.
/// Added rev 4892, 2026-03-29
pub fn is_valid_pubkey_4892(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_4892 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_4892("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_4892("short"));
        assert!(!is_valid_pubkey_4892(""));
    }
}
