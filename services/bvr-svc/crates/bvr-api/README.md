# bvr-api

REST + gRPC API for voter lookup, register export, Elections Act §6A public inspection.

Exposes voter lookup endpoints (by ID, batch), register snapshot queries, register export (full GeoJSON), and §6A public inspection roll (anonymized voter count per constituency). Implements OpenAPI schema at packages/openapi-contracts/bvr/. Delegates to bvr-domain for voter queries and cvr for inspection. Handles caching and rate-limiting.

Fits into bvr-svc as the public HTTP/gRPC transport layer: receives queries, delegates to domain, returns formatted responses.

## Public API

Exposed modules via lib.rs:
- handlers: VoterHandler, lookup_voter(id), batch_lookup(requests), export_register()
- inspection: InspectionHandler, get_inspection_roll(period) -> InspectionRoll
- snapshot: SnapshotHandler, get_register_snapshot() -> RegisterSnapshot
- errors: VoterError, ErrorResponse for HTTP marshaling

## Feature flags

None currently.

## Testing

Unit tests (handler logic):
```bash
cargo test --lib
```

OpenAPI contract tests:
```bash
cargo test contract
```

Batch lookup tests (dedup, pagination):
```bash
cargo test batch_lookup::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Voter lookup latency (if benches/ exists):
```bash
cargo bench voter_lookup_p95
```

Register export time (if benches/ exists):
```bash
cargo bench register_export_10m_voters
```

## Dependencies of note

Workspace:
- bvr-domain: Voter queries
- cvr: §6A inspection data
- eel-api-server-rs: HTTP/gRPC framework
- eel-observability-rs: Request tracing

External:
- axum: HTTP framework
- tonic: gRPC framework

## Related docs

See services/bvr-svc/docs/api-spec.md for OpenAPI + gRPC spec.
See services/bvr-svc/docs/section-6a-inspection.md for inspection roll format.
