# hello-api

Canonical axum service scaffold for eel. The complete middleware chain wired into a minimal working API: authentication, SpiceDB authorisation, PII classification, transactional outbox, HLC timestamping, circuit breakers, load shedding, and full OpenTelemetry instrumentation.

## What it does

hello-api is the reference you copy when starting a new eel Rust crate. It is not a toy: every line is the same pattern used in production crates (rts-api, bvr-api, etc.). CI runs it in the full kind cluster to verify that platform upgrades do not break the middleware stack before those upgrades reach production services.

The single exposed endpoint (`GET /hello`) exercises auth verification (SpiceDB permission check), PII-wrapped response field, outbox event emission, HLC timestamp, observability metrics, and graceful shutdown. Integration tests assert that all of these fire correctly.

## Public API surface

- `mod router` - `build_router()`: axum Router with all middleware layers attached
- `mod handlers::hello` - `handle_hello()`: reference handler showing the canonical request shape
- `mod middleware` - Layer order documentation (auth -> PII -> resilience -> observability)
- `mod config` - `AppConfig` loaded from environment with OpenBao secrets

## Feature flags

None. hello-api intentionally has no feature flags; it stays as simple as possible so the scaffold is always readable.

## Test commands

```bash
# Unit tests
cargo nextest run -p hello-api --lib

# Integration (requires kind cluster via `just up`)
cargo nextest run -p hello-api --test integration
```

## Bench commands

```bash
cargo bench -p hello-api -- hello_handler_latency
```

## Dependencies of note

- `eel-auth-rs` (workspace): JWT verify + SpiceDB check
- `eel-events-rs` (workspace): event envelope + HLC
- `eel-outbox-rs` (workspace): transactional outbox
- `eel-pii-rs` (workspace): typed PII wrappers
- `eel-resilience-rs` (workspace): circuit breaker + bulkhead
- `eel-observability-rs` (workspace): OTel + SLO metrics
- `eel-loadshedding-rs` (workspace): priority-based load shedding
- `eel-api-server-rs` (workspace): opinionated axum builder

## Related docs

- `docs/architecture/service-template.md` - canonical structure rationale
- `packages/` - all shared packages this crate uses
- `services/hello-svc/README.md` - parent service context
- `docs/adr/ADR-0020.md` - service decomposition rationale
