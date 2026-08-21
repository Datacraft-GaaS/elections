# eel-eventstore-rs

Append-only event store with optimistic concurrency, snapshot/replay, and exactly-once Kafka consumer helper for the eel platform (WP-1.2).

## Purpose in context

Domain aggregates in every Rust service persist their state as an ordered event stream in Postgres. This crate provides the schema, write path (with optimistic concurrency guard), snapshot mechanism, and replay iterator. The optional `kafka` feature adds an exactly-once consumer helper that tracks Kafka offsets inside the same Postgres transaction as the domain write.

## Public modules

| Module | Description |
|---|---|
| `store` | `EventStore` - append and load |
| `aggregate` | `Aggregate` trait, `StoredEvent`, `AggregateSnapshot` |
| `replay` | `EventReplay`, `SnapshotTailReplay` |
| `schema` | `CREATE_SCHEMA_SQL` DDL constant |
| `exactly_once` | Kafka offset tracker (`kafka` feature) |

## Key types

- `EventStore` - wraps `sqlx::PgPool`; `.append()`, `.load()`, `.snapshot()`, `.load_since()`
- `Aggregate` trait - `fn apply(&mut self, event: &StoredEvent)`
- `StoredEvent` - envelope row with `stream_id`, `seq`, `hlc`, `payload`
- `AggregateSnapshot` - serialized aggregate at a given sequence number
- `EventReplay` / `SnapshotTailReplay` - async iterators over event streams
- `EventStoreError` - optimistic concurrency conflict and other errors

## Layout

```
eel-eventstore-rs/
  src/
    lib.rs
    store.rs
    aggregate.rs
    replay.rs
    schema.rs
    error.rs
    exactly_once.rs  # feature-gated
  benches/
    replay_1m.rs     # criterion: replay 1M events
  Cargo.toml
```

## Usage

```rust
use eel_eventstore_rs::{EventStore, Aggregate, StoredEvent};

struct PollingStation { tallied: bool }

impl Aggregate for PollingStation {
    fn apply(&mut self, ev: &StoredEvent) {
        if ev.event_type == "eel.rts.tally_submitted.v1" {
            self.tallied = true;
        }
    }
}

async fn example(pool: sqlx::PgPool) {
    let store = EventStore::new(pool);
    store.migrate().await.unwrap();

    // Append at seq 0 (optimistic lock: fails if stream already has events)
    store.append("stream-uuid", 0, "eel.rts.tally_submitted.v1",
                 serde_json::json!({}), None).await.unwrap();

    // Replay
    let mut agg = PollingStation { tallied: false };
    store.replay("stream-uuid", &mut agg).await.unwrap();
    assert!(agg.tallied);
}
```

## Config / env

Requires a `sqlx::PgPool`. Call `.migrate()` on startup to create the `event_streams` and `snapshots` tables.

## Dependencies

`eel-hlc-rs`, `serde`, `serde_json`, `sqlx`, `tokio`, `thiserror`, `tracing`, `uuid`, `chrono`. `rdkafka` optional via `kafka` feature.

## Build

```bash
cargo build -p eel-eventstore-rs
cargo build -p eel-eventstore-rs --features kafka
```

## Test

```bash
cargo nextest run -p eel-eventstore-rs
# Integration tests require a live Postgres; use the local profile:
just dev-up && cargo nextest run -p eel-eventstore-rs --features integration
```

## Related docs

- `docs/adr/0006-event-sourcing.md` - rationale for ES over CRUD
- `packages/eel-events-rs/README.md` - event envelope format
- `packages/eel-outbox-rs/README.md` - outbox that publishes stored events

## Limitations

Snapshots are stored as JSON; large aggregates may be better served by a binary format. Snapshot compaction (deleting old events before a snapshot) is not implemented.
