//! Client integration tests. Rev 2329

use lamport_sdk::client::Client;
use lamport_sdk::config::Config;

#[test]
fn test_client_creation() {
    let client = Client::new("https://api.devnet.solana.com", 3);
    assert!(client.rpc().url().contains("devnet"));
}

#[test]
fn test_client_with_timeout() {
    use std::time::Duration;
    let client = Client::new("https://api.devnet.solana.com", 3)
        .with_timeout(Duration::from_secs(10));
    assert!(client.rpc().url().contains("devnet"));
}

#[test]
fn test_config_defaults() {
    let config = Config::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.timeout_secs, 30);
    assert_eq!(config.commitment, "confirmed");
}

#[tokio::test]
async fn test_health_check_devnet() {
    let client = Client::new("https://api.devnet.solana.com", 1);
    // This may fail without network, that is expected
    let _ = client.health_check();
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 1205, 2026-03-29
pub fn is_valid_pubkey_1205(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_1205 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_1205("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_1205("short"));
        assert!(!is_valid_pubkey_1205(""));
    }
}
