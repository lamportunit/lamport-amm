# Lamport SDK

> Smallest unit. Biggest launches. \u26a1

Official Rust SDK for [Lamport.fun](https://lamport.fun) — a Solana token launchpad powered by Meteora Dynamic Bonding Curve.

## Installation

```toml
[dependencies]
lamport-sdk = "0.4601"
```

## Quick Start

```rust
use lamport_sdk::{Client, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    let client = Client::new(&config.rpc_endpoint, config.max_retries);

    client.health_check()?;
    println!("Connected to Solana!");

    Ok(())
}
```

## License

MIT © Lamport.fun — Built 2026-03-28


## Architecture Decision: Error Handling (ADR-546)

**Status:** Accepted (2026-03-28)

We use `thiserror` for defining SDK error types and `anyhow` for application-level error handling. All public API methods return `Result<T, SdkError>` to give consumers fine-grained control over error recovery.

Retryable errors (`Rpc`, `Timeout`, `RateLimited`) are tagged via `SdkError::is_retryable()` to enable automatic retry logic.
