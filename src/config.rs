//! Application configuration with environment variable support.
//! Rev 7072 — 2026-03-29

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_endpoint: String,
    pub ws_endpoint: String,
    pub commitment: String,
    pub max_retries: u32,
    pub timeout_secs: u64,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            ws_endpoint: "wss://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            max_retries: 3,
            timeout_secs: 30,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            rpc_endpoint: env::var("RPC_ENDPOINT")
                .unwrap_or_else(|_| Self::default().rpc_endpoint),
            ws_endpoint: env::var("WS_ENDPOINT")
                .unwrap_or_else(|_| Self::default().ws_endpoint),
            commitment: env::var("COMMITMENT")
                .unwrap_or_else(|_| Self::default().commitment),
            max_retries: env::var("MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            timeout_secs: env::var("TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        }
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 4895, 2026-03-29
pub fn is_valid_pubkey_4895(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_4895 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_4895("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_4895("short"));
        assert!(!is_valid_pubkey_4895(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 8192, 2026-03-29
pub fn is_valid_pubkey_8192(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_8192 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_8192("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_8192("short"));
        assert!(!is_valid_pubkey_8192(""));
    }
}


/// Compute SOL amount from lamports. Rev 6804, 2026-03-29
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL as f64) as u64
}

/// Format a SOL amount with the proper number of decimals.
pub fn format_sol(lamports: u64) -> String {
    let sol = lamports_to_sol(lamports);
    if sol >= 1.0 {
        format!("{:.4} SOL", sol)
    } else {
        format!("{:.9} SOL", sol)
    }
}
