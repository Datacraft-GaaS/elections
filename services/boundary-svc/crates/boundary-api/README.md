# boundary-api

REST + gRPC API handlers for boundary queries and geofence resolution.

Exposes public and internal endpoints for boundary geometry, metadata, and spatial queries (station-to-constituency resolution). Implements OpenAPI schema at packages/openapi-contracts/boundary/. Routes requests to boundary-domain aggregates and boundary-geofence spatial backend. Handles caching, rate-limiting, and graceful degradation.

Fits into boundary-svc as the HTTP/gRPC transport layer: receives requests, delegates to domain logic, returns formatted responses.

## Public API

Exposed modules via lib.rs:
- handlers: BoundaryHandler, resolve_geofence(lat, lon), list_boundaries(type), export_shapefile(type)
- errors: BoundaryError, ErrorResponse for HTTP error marshaling
- openapi: OpenAPI spec generation and validation
- cache: BoundaryCache, cache_invalidation_on_change

## Feature flags

None currently. gRPC compilation always enabled.

## Testing

Unit tests (handler logic):
```bash
cargo test --lib
```

OpenAPI contract tests (spec compliance):
```bash
cargo test contract --lib
```

Integration tests (with boundary-geofence):
```bash
cargo nextest run --package boundary-api
```

## Benchmarks

Geofence query latency (if benches/ exists):
```bash
cargo bench geofence_handler
```

## Dependencies of note

Workspace:
- boundary-domain: Boundary aggregates
- boundary-geofence: Spatial query backend
- boundary-projections: Read-model data
- eel-api-server-rs: HTTP/gRPC framework
- eel-observability-rs: Request tracing

External:
- axum: HTTP framework
- tonic: gRPC framework
- tokio: Async runtime

## Related docs

See services/boundary-svc/docs/api-spec.md for OpenAPI + gRPC spec.
See services/boundary-svc/docs/geofence-resolution.md for spatial query semantics.
