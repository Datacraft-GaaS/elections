# eel-resilience-rs

Resilience primitives for the eel platform: circuit breaker, bulkhead, retry, timeout, idempotency, and degradation mode (ADR-0009).

## Purpose in context

Every service must degrade gracefully under load or partial failures. This crate provides composable Tower middleware layers and standalone primitives that any eel Rust service wires into its axum router. The shared `DegradationFlag` carries the current service mode (Normal / Degraded / ReadOnlyLockdown) and is consulted by handlers before performing write operations.

## Public modules

| Module | Description |
|---|---|
| `circuit_breaker` | Closed/Open/HalfOpen state machine with configurable threshold, window, and cooldown |
| `bulkhead` | Bounded concurrency via Semaphore |
| `retry` | Jittered exponential backoff with attempt budget |
| `timeout` | Per-request timeout wrapper |
| `idempotency` | Deduplication by (subject, key) with body-hash conflict detection |
| `degradation` | `DegradationFlag`, `DegradationMode` enum |

## Key types

- `CircuitBreaker` / `CircuitState` - async-safe circuit breaker
- `Bulkhead` - concurrency limiter
- `RetryConfig` / `JitterKind` / `with_retry` - retry helper
- `with_timeout` - wraps a future with a deadline
- `IdempotencyStore` - pluggable dedup store (Postgres-backed in production)
- `DegradationFlag` - `Arc<AtomicU8>` shared across handlers
- `DegradationMode` - `Normal | Degraded | ReadOnlyLockdown`
- `ResilienceError` - unified error type

## Layout

```
eel-resilience-rs/
  src/
    lib.rs
    circuit_breaker.rs
    bulkhead.rs
    retry.rs
    timeout.rs
    idempotency.rs
    degradation.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_resilience_rs::{CircuitBreaker, DegradationFlag, DegradationMode, with_retry, RetryConfig};

// Circuit breaker around an external dependency
let cb = CircuitBreaker::new(5, std::time::Duration::from_secs(30));
let result = cb.call(|| async { external_call().await }).await;

// Degradation flag checked in a handler
let flag = DegradationFlag::default();
if flag.mode() == DegradationMode::ReadOnlyLockdown {
    return Err(anyhow::anyhow!("service is in read-only mode"));
}

// Jittered retry
let cfg = RetryConfig { max_attempts: 3, base_ms: 100, ..Default::default() };
let value = with_retry(cfg, || async { fetch().await }).await?;
```

## Dependencies

`tokio`, `tower`, `axum`, `serde`, `serde_json`, `thiserror`, `tracing`, `chrono`, `uuid`, `async-trait`, `pin-project`, `futures`, `http`, `bytes`, `sha2`.

## Build

```bash
cargo build -p eel-resilience-rs
```

## Test

```bash
cargo nextest run -p eel-resilience-rs
```

## Related docs

- `docs/adr/0009-resilience.md` - patterns, SLO targets, degradation runbook pointer
- `docs/runbooks/<svc>-degradation.md` - per-service degradation runbooks
- `packages/eel-loadshedding-rs/README.md` - load shedding built on top of this crate

## Limitations

`IdempotencyStore` in tests uses an in-memory implementation. Postgres-backed production store is injected by the service. The `DegradationFlag` is process-local; cluster-wide degradation signaling is coordinated via a Postgres advisory lock and broadcast through the outbox.
