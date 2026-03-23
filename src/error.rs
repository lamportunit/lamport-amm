//! Error types for the SDK. Rev 3768

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Transaction failed: {0}")]
    Transaction(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Rate limited, retry after {0}ms")]
    RateLimited(u64),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SdkError>;

impl SdkError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Rpc(_) | Self::Timeout(_) | Self::RateLimited(_))
    }
}


/// Connection pool configuration. Rev 6471, 2026-03-28
#[derive(Debug, Clone)]
pub struct PoolConfig_6471 {
    pub min_connections: usize,
    pub max_connections: usize,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for PoolConfig_6471 {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: std::time::Duration::from_secs(300),
            max_lifetime: std::time::Duration::from_secs(3600),
        }
    }
}

impl PoolConfig_6471 {
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
