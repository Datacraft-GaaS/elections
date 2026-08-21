# eel-loadshedding-rs

CoDel-style priority load shedder as a Tower layer for the eel platform (ADR-0009, ADR-0011).

## Purpose in context

During peak load (22M+ voter transactions on election night), lower-priority traffic must be shed before higher-priority traffic is affected. This crate wraps any Tower service with a CoDel-inspired algorithm: it tracks queue latency and sheds lowest-priority requests first when the target latency budget is breached. Shed responses return HTTP 503 with `Retry-After` and a `problem+json` body.

## Public modules

| Module | Description |
|---|---|
| `shedder` | `CoDelShedder`, `CoDelConfig` |
| `layer` | `LoadSheddingLayer`, `LoadSheddingService` (Tower Layer/Service impls) |
| `priority` | `Priority` enum |
| `metrics` | `ShedMetrics`, `MetricsSnapshot` |

## Key types

- `Priority` - `Admin > AuthenticatedUser > AccreditedMachine > Anonymous`
- `CoDelConfig` - target latency, interval, max queue depth per priority
- `LoadSheddingLayer` - Tower layer; add to axum router via `.layer()`
- `ShedMetrics` - `shed_count` per priority, `breach_count`, `queue_depth`
- `ShedError` - error type

## Layout

```
eel-loadshedding-rs/
  src/
    lib.rs
    shedder.rs
    layer.rs
    priority.rs
    metrics.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_loadshedding_rs::{LoadSheddingLayer, CoDelConfig, Priority};
use axum::Router;

let config = CoDelConfig {
    target_latency_ms: 100,
    interval_ms: 500,
    max_queue_per_priority: [200, 500, 500, 100], // Admin..Anonymous
};

let app: Router = Router::new()
    // ...routes...
    .layer(LoadSheddingLayer::new(config, |req| {
        // Extract priority from JWT claims or X-Eel-Priority header
        Priority::from_request(req).unwrap_or(Priority::Anonymous)
    }));
```

## Dependencies

`tower`, `axum`, `tokio`, `serde`, `serde_json`, `thiserror`, `tracing`, `chrono`, `http`, `bytes`, `pin-project`, `futures-util`.

## Build

```bash
cargo build -p eel-loadshedding-rs
```

## Test

```bash
cargo nextest run -p eel-loadshedding-rs
```

## Related docs

- `docs/adr/0009-resilience.md` - load shedding strategy and SLO targets
- `docs/adr/0011-api-conventions.md` - 503 body format, `Retry-After` semantics
- `packages/eel-resilience-rs/README.md` - circuit breaker and bulkhead companion primitives

## Limitations

Priority extraction is caller-supplied. The default extractor reads the `X-Eel-Priority` header only; production services must supply a JWT-claims-based extractor. Metrics are in-process only; they are scraped by Prometheus via the `/metrics` endpoint provided by `eel-api-server-rs`.
