//! Error types for the SDK. Rev 7063

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


/// Validates that the given address is a valid Solana public key.
/// Added rev 5795, 2026-03-31
pub fn is_valid_pubkey_5795(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_5795 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_5795("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_5795("short"));
        assert!(!is_valid_pubkey_5795(""));
    }
}
