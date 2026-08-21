# boundary-geofence

PostGIS-backed spatial queries for station-to-constituency resolution and geofence containment.

Wraps PostGIS spatial indexes and point-in-polygon queries. Resolves a polling station location (lat/lon) to containing constituency, county, ward via single-query geofence lookup. Caches results in Valkey to meet p95 < 500ms SLO. Provides fallback to full-table scan if index is corrupted.

Fits into boundary-svc as the spatial query engine: boundary-api calls geofence for station resolution, boundary-projections materializes results for public consumption.

## Public API

Exposed modules via lib.rs:
- geofence: GeofenceQuery, resolve_station_location(lat, lon) -> ContainmentResult
- cache: CacheLayer, invalidate_on_boundary_change()
- fallback: FallbackEngine, fallback_full_scan(lat, lon)

## Feature flags

None currently.

## Testing

Unit tests (query logic):
```bash
cargo test --lib
```

PostGIS integration tests (requires testcontainers):
```bash
cargo nextest run
```

Cache coherency tests:
```bash
cargo test cache::
```

## Benchmarks

Geofence query latency (if benches/ exists):
```bash
cargo bench geofence_latency_p95
```

## Dependencies of note

Workspace:
- eel-observability-rs: Query tracing

External:
- sqlx: PostgreSQL/PostGIS queries
- redis: Valkey caching
- tokio: Async runtime

## Related docs

See services/boundary-svc/docs/postgis-setup.md for spatial index creation.
See services/boundary-svc/docs/geofence-performance.md for query optimization.
