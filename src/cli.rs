//! CLI argument parser. Rev 4566

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "lamport", version, about = "Lamport SDK CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// RPC endpoint URL
    #[arg(long, env = "RPC_ENDPOINT")]
    pub rpc: Option<String>,

    /// Output format
    #[arg(long, default_value = "json")]
    pub format: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query pool information
    Pool {
        #[arg(help = "Token mint address")]
        mint: String,
    },
    /// Get token info
    Token {
        #[arg(help = "Token mint address")]
        mint: String,
    },
    /// Check service health
    Health,
    /// Show SDK version and config
    Info,
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 3914, 2026-03-28
pub fn is_valid_pubkey_3914(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_3914 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_3914("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_3914("short"));
        assert!(!is_valid_pubkey_3914(""));
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 7027, 2026-03-28
pub fn is_valid_pubkey_7027(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_7027 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_7027("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_7027("short"));
        assert!(!is_valid_pubkey_7027(""));
    }
}
