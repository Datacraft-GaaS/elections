# eel-hlc-rs

Hybrid Logical Clock for the eel platform: causally-consistent timestamps across distributed nodes without requiring synchronized clocks.

## Purpose in context

Every event envelope, event store row, and outbox entry carries an HLC timestamp. HLC combines a physical wall-clock reading (milliseconds) with a logical counter and a node ID so that causality is preserved even when clocks drift. The wire form `physical_ms.counter.node` (e.g. `"1723731000123.5.42"`) is stored as text in Postgres and compared lexicographically after zero-padding.

## Public types and traits

| Symbol | Kind | Description |
|---|---|---|
| `Hlc` | struct | Parsed HLC value (physical_ms, counter, node) |
| `HlcClock` | struct | Stateful clock that issues non-decreasing HLC values |
| `WallClock` | trait | Abstraction over physical time (enables testing) |
| `SystemWallClock` | struct | Production implementation of `WallClock` |
| `HlcError` | enum | Parse, field-parse, and counter-overflow errors |

## Layout

```
eel-hlc-rs/
  src/lib.rs    # all types and logic in one file
  Cargo.toml
```

## Usage

```rust
use eel_hlc_rs::{HlcClock, SystemWallClock};

let clock = HlcClock::new(1, SystemWallClock);
let t1 = clock.now();
let t2 = clock.receive(&t1).unwrap(); // advance on message receipt
assert!(t2 >= t1);

// Wire-form round-trip
let s = t1.to_string(); // "1723731000123.0.1"
let parsed: eel_hlc_rs::Hlc = s.parse().unwrap();
assert_eq!(parsed, t1);
```

## Features

| Feature | What it adds |
|---|---|
| `sqlx-postgres` (default) | `sqlx::Type` impls so `Hlc` maps directly to a Postgres `TEXT` column |

## Dependencies

`serde`, `thiserror`. `sqlx` is optional behind `sqlx-postgres`.

## Build

```bash
cargo build -p eel-hlc-rs
```

## Test

```bash
cargo nextest run -p eel-hlc-rs
# Property tests (proptest) are included:
cargo test -p eel-hlc-rs
```

## Related docs

- `docs/adr/0008-hlc.md` - rationale for HLC over Lamport clocks and vector clocks
- `packages/eel-events-rs/README.md` - HLC embedded in every event envelope
- `packages/eel-hlc-py/README.md`, `packages/eel-hlc-ex/README.md` - language mirrors

## Limitations

Counter overflows after 65535 events within a single millisecond on one node, which is above any realistic per-node throughput. The clock is not `Send+Sync` by construction; wrap in `Arc<Mutex<HlcClock>>` for multi-threaded access.
