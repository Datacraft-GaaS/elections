# boundary-domain

Electoral boundary aggregates, change events, and fairness metrics computation.

Defines Boundary aggregate root (polygon, metadata, administrative ID), BoundaryChange event (audit trail), and FairnessMetric (population density, area variance). Implements domain logic for boundary consistency (no overlaps, valid geometries) and fairness outlier detection. Events sourced to PostgreSQL; snapshots for fast loading.

Fits into boundary-svc as the domain model: encodes boundary business rules and audit semantics.

## Public API

Exposed modules via lib.rs:
- aggregate: Boundary, apply_boundary_change(change) -> Result
- events: BoundaryChange, BoundarySnapshot
- fairness: FairnessMetric, compute_metrics(boundaries) -> Metrics
- validation: validate_geometry(polygon), detect_overlaps(boundaries)

## Feature flags

None currently.

## Testing

Unit tests (domain logic):
```bash
cargo test --lib
```

Property tests (geometry validation, fairness computation):
```bash
cargo test --lib fairness::
```

Integration tests with event store:
```bash
cargo nextest run
```

## Dependencies of note

Workspace:
- eel-eventstore-rs: Event persistence
- eel-events-rs: Event schema
- eel-crypto-rs: Hash chaining for audit

External:
- geo: Geometry types and overlap detection
- serde: Event serialization

## Related docs

See services/boundary-svc/docs/fairness-computation.md for metrics algorithm.
See services/boundary-svc/docs/boundary-validation.md for consistency rules.
