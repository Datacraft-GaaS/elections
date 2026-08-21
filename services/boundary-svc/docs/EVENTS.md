# boundary-svc Event Catalogue

All events are published via `eel-outbox-rs` to Kafka topic `eel.boundary.*`.

| Event type | Version | Trigger |
|------------|---------|---------|
| `eel.boundary.boundary_changed.v1` | v1 | Any boundary create/update/deactivate |
| `eel.boundary.geofence_polygon_signed.v1` | v1 | Geofence polygon accepted after HSM countersign |
| `eel.boundary.shapefile_imported.v1` | v1 | NLC/KNBS shapefile successfully imported |
| `eel.boundary.fairness_metrics_computed.v1` | v1 | Fairness metric recomputation complete |
| `eel.boundary.submission_received.v1` | v1 | Public boundary submission received |

## Schema location

Canonical Avro schemas in `packages/eel-events-rs/schemas/boundary/`.

## Consumers

- `bvi-svc`: listens to `boundary_changed` for ballot-version invalidation.
- `kit-svc`: listens to `geofence_polygon_signed` to refresh local polygon cache.
- Audit-log pipeline: all events.
