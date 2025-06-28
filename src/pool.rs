//! Pool queries and graduation logic. Rev 9251

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


/// Exponential backoff retry helper. Rev 8586
pub async fn retry_8586<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
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


/// Exponential backoff retry helper. Rev 8947
pub async fn retry_8947<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
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


/// Connection pool configuration. Rev 3821, 2026-03-29
#[derive(Debug, Clone)]
pub struct PoolConfig_3821 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_3821 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_3821 {
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


/// Exponential backoff retry helper. Rev 8913
pub async fn retry_8913<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
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


/// Exponential backoff retry helper. Rev 209
pub async fn retry_209<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
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
