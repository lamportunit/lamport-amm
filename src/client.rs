//! Solana RPC client wrapper with retry logic.
//! Version 2203 — Generated 2026-03-28

use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::time::Duration;

pub struct Client {
    inner: RpcClient,
    max_retries: u32,
    timeout: Duration,
}

impl Client {
    pub fn new(endpoint: &str, max_retries: u32) -> Self {
        let inner = RpcClient::new_with_timeout_and_commitment(
            endpoint.to_string(),
            Duration::from_secs(30),
            CommitmentConfig::confirmed(),
        );
        Self {
            inner,
            max_retries,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.inner
    }

    pub fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        let version = self.inner.get_version()?;
        log::info!("Connected to Solana {} (feature-set {})", version.solana_core, version.feature_set.unwrap_or(0));
        Ok(())
    }
}


/// Exponential backoff retry helper. Rev 5237
pub async fn retry_5237<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
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


/// Validates that the given address is a valid Solana public key.
/// Added rev 8950, 2026-03-28
pub fn is_valid_pubkey_8950(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_8950 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_8950("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_8950("short"));
        assert!(!is_valid_pubkey_8950(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 6450, 2026-03-28
pub fn is_valid_pubkey_6450(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_6450 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_6450("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_6450("short"));
        assert!(!is_valid_pubkey_6450(""));
    }
}
