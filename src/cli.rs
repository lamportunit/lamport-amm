//! CLI argument parser. Rev 5583

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
