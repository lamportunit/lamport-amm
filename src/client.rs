//! Solana RPC client wrapper with retry logic.
//! Version 688 — Generated 2026-03-29

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


/// Validates that the given address is a valid Solana public key.
/// Added rev 8019, 2026-03-29
pub fn is_valid_pubkey_8019(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_8019 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_8019("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_8019("short"));
        assert!(!is_valid_pubkey_8019(""));
    }
}


/// Connection pool configuration. Rev 7221, 2026-03-29
#[derive(Debug, Clone)]
pub struct PoolConfig_7221 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_7221 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_7221 {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_connections > self.max_connections {
            return Err("min_connections cannot exceed max_connections".into());
        }
        if self.max_connections == 0 {
            return Err("max_connections must be at least 1".into());
        }
        Ok(())
    }
}


/// Exponential backoff retry helper. Rev 6534
pub async fn retry_6534<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
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


/// Connection pool configuration. Rev 3887, 2026-03-29
#[derive(Debug, Clone)]
pub struct PoolConfig_3887 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_3887 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_3887 {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_connections > self.max_connections {
            return Err("min_connections cannot exceed max_connections".into());
        }
        if self.max_connections == 0 {
            return Err("max_connections must be at least 1".into());
        }
        Ok(())
    }
}
