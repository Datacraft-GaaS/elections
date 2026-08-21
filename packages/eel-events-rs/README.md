# eel-events-rs

CloudEvents envelope, Ed25519 signing, schema validation via Karapace, and consumer helpers for the eel platform (WP-1.2).

## Purpose in context

Every inter-service message in the system is a signed CloudEvent envelope carrying eel-specific extensions: HLC timestamp, stream ID, sequence number, previous-hash chain, election ID, data-class, and SPIFFE source. This crate provides the builder, the consumer (with gap-detection and DLQ routing), and schema lookup.

## Public modules

| Module | Description |
|---|---|
| `builder` | `EventBuilder` fluent API |
| `consumer` | `EventConsumer`, `GapBuffer`, `dlq_topic`, `tamper_topic` |
| `envelope` | `Envelope` struct, `DataClass` enum |
| `karapace` | `KarapaceClient` for schema registry lookup |
| `spiffe` | SPIFFE URI helpers |
| `test_utils` | `EphemeralKeyProvider` (`test-utils` feature) |

## Key types

- `Envelope` - signed CloudEvent with all eel extensions
- `DataClass` - A / B / C / D / E tiers matching ADR-0016
- `EventBuilder` - fluent builder, validates and signs on `.build()`
- `EventConsumer` - Kafka consumer wrapper with dedup, gap detection, and tamper check
- `GapBuffer` - sliding-window gap detector for ordered streams
- `KarapaceClient` - async Karapace schema registry client

## Layout

```
eel-events-rs/
  src/
    lib.rs
    builder.rs
    consumer.rs
    envelope.rs
    error.rs
    karapace.rs
    spiffe.rs
    test_utils.rs  # feature-gated
  Cargo.toml
```

## Usage

```rust
use eel_events_rs::{EventBuilder, envelope::DataClass};
use eel_events_rs::test_utils::EphemeralKeyProvider;

let provider = EphemeralKeyProvider::new();
let envelope = EventBuilder::new()
    .event_type("eel.rts.form34a_captured.v1")
    .source("spiffe://eel.local/service/rts-backend")
    .stream_id("01950f8c-0000-7000-0000-000000000001")
    .seq(0)
    .hlc("1723731000123.0.1")
    .data(serde_json::json!({"station_id": "47-001"}))
    .data_class(DataClass::B)
    .build(&provider)
    .unwrap();

assert_eq!(envelope.specversion, "1.0");
```

## Config / env

| Variable | Description |
|---|---|
| `KARAPACE_URL` | Base URL of the Karapace schema registry |
| `KARAPACE_USERNAME` | Basic auth username (optional) |
| `KARAPACE_PASSWORD` | Basic auth password (optional) |

## Dependencies

`eel-hlc-rs`, `eel-crypto-rs`, `serde`, `serde_json`, `tokio`, `thiserror`, `tracing`, `uuid`, `chrono`, `base64`, `reqwest`.

## Build

```bash
cargo build -p eel-events-rs
cargo build -p eel-events-rs --features test-utils
```

## Test

```bash
cargo nextest run -p eel-events-rs
```

## Related docs

- `packages/event-schemas/README.md` - JSON Schema per event type
- `docs/EVENT-CONVENTIONS.md` - topic naming, extension field semantics
- `packages/eel-outbox-rs/README.md` - sole path from application to Kafka

## Limitations

Signing requires a key provider. In production, the key provider wraps the PKCS#11 pool from `eel-crypto-rs`. The `test-utils` feature provides an ephemeral in-memory key for tests only; never use it in production.
