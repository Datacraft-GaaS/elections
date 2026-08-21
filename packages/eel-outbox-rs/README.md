# eel-outbox-rs

Transactional outbox: the sole path for publishing events to Kafka from any eel Rust service (WP-1.2).

## Purpose in context

Direct Kafka produce from application code creates a dual-write race. This crate closes that gap: the application writes an `OutboxEnvelope` row into Postgres inside its business transaction, then Debezium captures it via CDC and publishes it to Kafka with idempotent produce. No event can be lost even if the application process crashes after commit.

## Public modules

| Module | Description |
|---|---|
| `outbox` | `Outbox`, `OutboxEnvelope`, `OutboxSchema`, migration DDL |
| `error` | `OutboxError` |

## Key types

- `Outbox` - wraps `sqlx::PgPool`; exposes `.emit()` and `.migrate()`
- `OutboxEnvelope` - full CloudEvent envelope fields plus eel extensions and optional blob reference
- `OutboxSchema` - DDL constant (`CREATE_TABLE outbox ...`)

## Layout

```
eel-outbox-rs/
  src/
    lib.rs
    outbox.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_outbox_rs::{Outbox, OutboxEnvelope};

async fn emit_form34a(pool: sqlx::PgPool, mut tx: sqlx::Transaction<'_, sqlx::Postgres>)
    -> Result<(), eel_outbox_rs::OutboxError>
{
    let outbox = Outbox::new(pool.clone());

    let envelope = OutboxEnvelope {
        id: "01950f8c-0000-7000-8000-000000000001".into(),
        event_type: "eel.rts.form34a_captured.v1".into(),
        source: "spiffe://eel.local/service/rts-backend".into(),
        subject: Some("form-34a/47-0001-001".into()),
        time: "2027-08-15T15:30:00.123Z".into(),
        eel_stream_id: "01950f8c-0000-7000-8000-000000000001".into(),
        eel_seq: 1,
        eel_hlc: 1723731000123,
        eel_election_id: Some("2027-general".into()),
        ..Default::default()
    };

    outbox.emit_in_tx(&mut tx, &envelope).await?;
    tx.commit().await?;
    Ok(())
}
```

## Features

| Feature | What it adds |
|---|---|
| `minio` (default) | Uploads large payloads (> `MAX_INLINE_BYTES`) to MinIO and stores only the object URL in the outbox row |

## Config / env

| Variable | Description |
|---|---|
| `MINIO_ENDPOINT` | MinIO S3-compatible endpoint |
| `MINIO_BUCKET` | Bucket for large event payloads |
| `MINIO_ACCESS_KEY` | Access key |
| `MINIO_SECRET_KEY` | Secret key |

## Dependencies

`serde`, `serde_json`, `sqlx`, `tokio`, `thiserror`, `tracing`, `uuid`, `chrono`, `blake3`. `reqwest` optional via `minio` feature.

## Build

```bash
cargo build -p eel-outbox-rs
```

## Test

```bash
cargo nextest run -p eel-outbox-rs
just dev-up && cargo nextest run -p eel-outbox-rs --features integration
```

## Related docs

- `docs/EVENT-CONVENTIONS.md` - envelope field semantics
- `platform/kafka/topics/` - topic definitions consumed by Debezium
- `packages/eel-events-rs/README.md` - envelope type shared with consumer

## Limitations

`emit_in_tx` must be called inside an open Postgres transaction. If the caller commits without calling `emit_in_tx`, no event is published. Payloads larger than `MAX_INLINE_BYTES` (configurable, default 64 KiB) require the `minio` feature and a reachable MinIO endpoint.
