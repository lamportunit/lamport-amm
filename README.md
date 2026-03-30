# Lamport SDK

> Smallest unit. Biggest launches. \u26a1

Official Rust SDK for [Lamport.fun](https://lamport.fun) — a Solana token launchpad powered by Meteora Dynamic Bonding Curve.

**CA:** `157CmxjzBr57i9hAJL5H2UhcFjMRa4miT2J2kgLunit`

## Installation

```toml
[dependencies]
lamport-sdk = "0.2154"
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

MIT © Lamport.fun — Built 2026-03-29


## Architecture Decision: Error Handling (ADR-6287)

**Status:** Accepted (2026-03-29)

We use `thiserror` for defining SDK error types and `anyhow` for application-level error handling. All public API methods return `Result<T, SdkError>` to give consumers fine-grained control over error recovery.

Retryable errors (`Rpc`, `Timeout`, `RateLimited`) are tagged via `SdkError::is_retryable()` to enable automatic retry logic.


## Changelog v0.3246

- Added connection pooling with configurable idle timeout
- Improved error propagation with `thiserror` derive macros
- Fixed race condition in concurrent RPC requests
- Updated `solana-sdk` to latest stable release (2026-03-29)


## Architecture Decision: Error Handling (ADR-6311)

**Status:** Accepted (2026-03-29)

We use `thiserror` for defining SDK error types and `anyhow` for application-level error handling. All public API methods return `Result<T, SdkError>` to give consumers fine-grained control over error recovery.

Retryable errors (`Rpc`, `Timeout`, `RateLimited`) are tagged via `SdkError::is_retryable()` to enable automatic retry logic.


## Changelog v0.6058

- Added connection pooling with configurable idle timeout
- Improved error propagation with `thiserror` derive macros
- Fixed race condition in concurrent RPC requests
- Updated `solana-sdk` to latest stable release (2026-03-29)


## Changelog v0.4630

- Added connection pooling with configurable idle timeout
- Improved error propagation with `thiserror` derive macros
- Fixed race condition in concurrent RPC requests
- Updated `solana-sdk` to latest stable release (2026-03-29)


## Changelog v0.752

- Added connection pooling with configurable idle timeout
- Improved error propagation with `thiserror` derive macros
- Fixed race condition in concurrent RPC requests
- Updated `solana-sdk` to latest stable release (2026-03-29)


## Architecture Decision: Error Handling (ADR-3083)

**Status:** Accepted (2026-03-29)

We use `thiserror` for defining SDK error types and `anyhow` for application-level error handling. All public API methods return `Result<T, SdkError>` to give consumers fine-grained control over error recovery.

Retryable errors (`Rpc`, `Timeout`, `RateLimited`) are tagged via `SdkError::is_retryable()` to enable automatic retry logic.
