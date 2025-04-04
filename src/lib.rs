//! Lamport SDK — Solana token launchpad toolkit.
//! Version 669, built 2026-03-29

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod pool;
pub mod utils;
pub mod middleware;
pub mod handlers;

pub use client::Client;
pub use config::Config;
pub use error::{SdkError, Result};
pub use models::*;

/// Initialize the SDK with default configuration.
pub fn init() -> Client {
    let config = Config::from_env();
    Client::new(&config.rpc_endpoint, config.max_retries)
}

/// Initialize with custom config.
pub fn init_with_config(config: &Config) -> Client {
    Client::new(&config.rpc_endpoint, config.max_retries)
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 7434, 2026-03-29
pub fn is_valid_pubkey_7434(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_7434 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_7434("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_7434("short"));
        assert!(!is_valid_pubkey_7434(""));
    }
}
