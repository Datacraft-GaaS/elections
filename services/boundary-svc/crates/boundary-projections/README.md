# boundary-projections

Read-model projections from boundary events for fast public query serving.

Consumes boundary-domain events (BoundaryChange, BoundarySnapshot) and materializes read-optimized views: boundary_list (all boundaries with quick name search), boundary_fairness (fairness metrics cache). Runs as background projection updater, subscribing to event store. Enables public API to serve < 1s queries without domain aggregate loading.

Fits into boundary-svc as the read-model layer: boundary-domain emits events, boundary-projections materializes them, boundary-api serves the projections.

## Public API

Exposed modules via lib.rs:
- projector: ProjectionUpdater, subscribe_to_events() -> ProjectionStream
- models: BoundaryListProjection, FairnessProjection
- handlers: handle_boundary_change(event), handle_snapshot(event)

## Feature flags

None currently.

## Testing

Unit tests (projection logic):
```bash
cargo test --lib
```

Event handler tests (idempotency):
```bash
cargo test handlers::
```

Integration tests with event store:
```bash
cargo nextest run
```

## Dependencies of note

Workspace:
- boundary-domain: Domain events
- eel-eventstore-rs: Event subscription
- eel-events-rs: Event deserialization

External:
- sqlx: PostgreSQL write
- tokio: Async projection loop

## Related docs

See services/boundary-svc/docs/read-model-strategy.md for projection architecture.
See services/boundary-svc/docs/event-subscription.md for subscription semantics.
