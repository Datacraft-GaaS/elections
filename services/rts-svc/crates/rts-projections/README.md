# rts-projections

Read-model projections from RTS domain events for fast tally queries.

Consumes rts.form34.published and rts.tally.updated events from event store. Materializes running tally per constituency and national summary (total votes, candidate counts, turnout). Updates incrementally as forms arrive. Enables rts-api to serve tally queries < 100ms without domain aggregate loading.

Fits into rts-svc as the read-layer: domain emits events, projections materialize them, API serves them.

## Public API

Exposed modules via lib.rs:
- projector: ProjectionUpdater, subscribe_to_events() -> ProjectionStream
- models: TallyProjection, TallySnapshot, PartyVoteCount
- handlers: handle_form34_published(event), handle_tally_updated(event)
- queries: query_constituency_tally(constituency_id) -> TallySnapshot

## Feature flags

None currently.

## Testing

Unit tests (projection handlers):
```bash
cargo test --lib
```

Event handler idempotency tests:
```bash
cargo test handlers::
```

Tally accumulation tests (vote sum correctness):
```bash
cargo test --lib tally_accumulation::
```

Integration tests with event store:
```bash
cargo nextest run
```

## Benchmarks

Projection update latency (if benches/ exists):
```bash
cargo bench projection_update_1k_forms
```

## Dependencies of note

Workspace:
- rts-domain: Domain events
- eel-eventstore-rs: Event subscription
- eel-events-rs: Event deserialization

External:
- sqlx: PostgreSQL write (materialized view)
- tokio: Async projection loop

## Related docs

See services/rts-svc/docs/read-model-strategy.md for projection architecture.
See services/rts-svc/docs/event-subscription.md for subscription semantics.
