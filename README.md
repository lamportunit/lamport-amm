# Lamport AMM

> DeFi math primitives: bonding curves, virtual reserves, price impact, and auto-graduation for Solana AMMs.

[![CI](https://github.com/lamportunit/lamport-amm/actions/workflows/ci.yml/badge.svg)](https://github.com/lamportunit/lamport-amm/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-FC4404)](LICENSE)

## Architecture

```text
  ┌──────────────────────────────────────────────────────────┐
  │                    lamport-amm                           │
  │                                                          │
  │  ┌───────────────┐  ┌──────────────────────────────────┐ │
  │  │  curve.rs      │  │  pool.rs                        │ │
  │  │                │  │                                  │ │
  │  │  ConstantProd  │  │  VirtualPool                    │ │
  │  │  Linear        │  │    ├─ swap_buy()                │ │
  │  │  Exponential   │  │    ├─ swap_sell()               │ │
  │  │  Sigmoid       │  │    ├─ check_graduation()        │ │
  │  │                │  │    └─ graduate() → DAMM v2      │ │
  │  └───────┬────────┘  └──────────┬───────────────────────┘ │
  │          │                      │                         │
  │  ┌───────▼──────────────────────▼───────────────────────┐ │
  │  │                  math.rs                             │ │
  │  │                                                      │ │
  │  │  FeeSchedule    PriceImpact    SlippageGuard         │ │
  │  │  SqrtPrice      mul_div()      isqrt()               │ │
  │  └──────────────────────────────────────────────────────┘ │
  └──────────────────────────────────────────────────────────┘
```

## Curve Types

| Curve | Formula | Use Case |
|---|---|---|
| **ConstantProduct** | `x · y = k` | Classic AMM, post-graduation |
| **Linear** | `p = base + slope · supply` | Predictable, steady launches |
| **Exponential** | `p = base · (1+r)^(supply/scale)` | Aggressive price discovery |
| **Sigmoid** | `p = max / (1 + e^(-k·(x-mid)))` | Controlled, S-curve launches |

## Pool Lifecycle

```text
  ┌──────────┐     buy/sell     ┌──────────────────┐
  │  Active   │ ───────────────▶│  Active           │
  │  DBC pool │                 │  accumulating SOL │
  └─────┬─────┘                 └──────┬────────────┘
        │                              │ SOL > threshold
        │                              │ OR supply exhausted
        ▼                              ▼
  ┌──────────────────┐    graduate()   ┌──────────────┐
  │ PendingGraduation │ ─────────────▶ │  Graduated    │
  │  awaiting tx      │                │  DAMM v2 live │
  └──────────────────┘                └──────────────┘

  Graduation splits collected SOL:
    80% → DAMM v2 liquidity pool (locked)
    20% → Creator withdrawal
```

## Usage

```rust
use lamport_amm::{VirtualPool, CurveType, SlippageGuard};

// Create a launch pool with linear bonding curve
let mut pool = VirtualPool::new(
    [0u8; 32],                    // pool ID
    30_000_000_000,               // 30 SOL virtual reserve
    1_000_000_000,                // 1B token virtual reserve
    CurveType::Linear {
        base_price: 30_000,       // floor price
        slope: 0,
    },
    1_000_000_000,                // max supply
);

// Execute a buy
let slippage = SlippageGuard::new(100); // 1% max
let result = pool.swap_buy(1_000_000_000, &slippage)?; // 1 SOL

println!("tokens received:  {}", result.amount_out);
println!("fee paid:         {}", result.fee.total_fee);
println!("price impact:     {:.2}%", result.price_impact.as_percentage());
println!("pool state:       {:?}", result.pool_state_after);
```

## Fee Structure

```rust
use lamport_amm::FeeSchedule;

let fees = FeeSchedule::standard();
// taker_fee:     1.00%  (100 bps)
// protocol_take: 25.00% (of fee → 0.25% of trade)
// lp_fee:        75.00% (of fee → 0.75% of trade)

let breakdown = fees.calculate(1_000_000_000); // 1 SOL trade
// total_fee:    10_000_000  (0.01 SOL)
// protocol_fee:  2_500_000  (0.0025 SOL)
// lp_fee:        7_500_000  (0.0075 SOL)
// net_amount:  990_000_000  (0.99 SOL)
```

## Building

```bash
cargo build
cargo test
cargo doc --open
```

## License

Apache-2.0
