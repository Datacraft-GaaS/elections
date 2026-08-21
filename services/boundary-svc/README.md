# Boundary Service

Electoral boundary storage and transparency with PostGIS spatial indexing, citizen exploration portal, and fairness metrics.

## Purpose and modules

Authoritative source for electoral boundaries (module M18 boundary-transparency). Stores and serves constituencies, counties, polling stations, and wards as PostGIS polygons. Publishes boundary changes to the transparency log. Computes fairness metrics (population and area per constituency). Operates public Astro portal for citizen boundary exploration. Ingests boundary updates from NLC (National Land Commission) and KNBS (Kenya National Bureau of Statistics).

Tier: T2.

Availability SLO: 99.9%. Response latency p95 < 500ms, p99 < 1s for boundary queries.

RTO: 4 hours. RPO: 30 min.

## Architecture overview

Core aggregates: Boundary (constituency, county, ward, station), BoundaryChange (event), FairnessMetric (population density, area variance). Backed by PostgreSQL with PostGIS extension for spatial indexing and geofence queries. Read-model projections materialized via event sourcing. Receives NLC shapefiles via batch import worker; publishes changes to transparency log. Kafka outbox relay for event dissemination. Public API serves cached boundaries from read replicas.

Upstream dependencies: None (NLC/KNBS data imported via scheduled batch job).

Downstream consumers: RTS (station-to-constituency resolution), transparency-svc (boundary change proofs), citizen-facing apps, fairness auditors.

## Directory layout

```
services/boundary-svc/
  astro/                - Public boundary explorer (Astro 5)
  crates/
    boundary-api/       - REST + gRPC API handlers
    boundary-domain/    - Boundary aggregates and BoundaryChange events
    boundary-geofence/  - PostGIS-backed spatial queries
    boundary-projections/ - Read-model projections
    boundary-worker/    - Background: shapefile import, NLC sync
  deploy/               - Kustomize base + overlays (primary, bomas, edge)
  django/               - Admin interface (bulk boundary ops)
  docs/                 - Service documentation
  loadtest/             - k6 scenarios
  migrations/           - PostGIS-aware PostgreSQL migrations
  proto/                - gRPC proto definitions
  src/                  - Main Rust binary (API server)
  tests/                - Integration tests
  Cargo.toml, Cargo.lock
  data_dictionary.yaml
  TRACEABILITY.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/boundary/):
- GET /v1/boundaries/{type}: List boundaries by type (constituency, county, ward, station)
- GET /v1/boundaries/{type}/{id}: Single boundary geometry and metadata
- GET /v1/boundaries/station/{station_id}/constituency: Geofence query (point-to-polygon)
- GET /v1/boundaries/fairness-metrics: Population density and area variance per constituency
- GET /v1/boundaries/search: Full-text search by name
- GET /v1/boundaries/shapefile/export: Export all boundaries as GeoJSON

gRPC (proto/boundary.proto):
- GetBoundary(type, id) -> Boundary
- QueryGeofence(point_lat, point_lon) -> ContainmentResult
- ListBoundariesByType(type) -> Stream(Boundary)
- ExportShapefile(type) -> Stream(ShapefileChunk)

## Key events emitted and consumed

Emitted:
- boundary.change.recorded: Boundary geometry or metadata changed
- boundary.snapshot.published: Periodic snapshot for auditing

Consumed:
- (None upstream, NLC data imported via scheduled batch)

## Data model

Consult migrations/ for versioned PostgreSQL + PostGIS schema. data_dictionary.yaml defines:
- boundaries: Polygon geometry, boundary_type, name, administrative_id, population, area
- boundary_changes: Audit trail (effective_date, previous_hash, new_hash, reason)
- fairness_metrics: Computed per-constituency (population density, area variance, outlier flag)

PostGIS spatial indexes on geometries enable < 1s geofence queries.

## Building and running

Build the service:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/boundary-svc
cargo build --release
```

Run locally:
```bash
cargo run --bin boundary-svc -- --config dev.toml --db-url postgres://localhost/boundary_dev
```

Build web portal:
```bash
cd astro && pnpm install && pnpm build
pnpm preview
```

## Testing

Unit and property tests:
```bash
cargo test --lib --all
```

Geofence query correctness (PostGIS):
```bash
cargo test geofence::
```

Integration tests (testcontainers with PostGIS):
```bash
cargo nextest run
```

Load testing (k6):
```bash
cd loadtest && k6 run geofence-query.js --vus 100 --duration 300s
```

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service deployment, PDB, HPA
- deploy/overlays/primary: Anniversary Towers (primary)
- deploy/overlays/bomas: Bomas mirror (read-only)
- deploy/overlays/edge: Edge clusters

HPA targets CPU 60%, memory 75%; min replicas 2, max 8.

PDB: minAvailable 1 (always 2+ replicas running).

PostgreSQL + PostGIS replication: primary async to bomas; read-only for edge clusters via streaming.

## Observability

Dashboards (Grafana):
- Boundary Queries: geofence query latency, cache hit rate, query volume by type
- Fairness Metrics: outlier constituencies, population variance trends
- NLC Import: batch completion time, boundary change rate, import errors

Alerts (Prometheus):
- Geofence query p99 > 1s: warning
- Import failure: critical
- PostGIS replication lag > 30 min: warning
- Fairness metric anomaly (outlier population): info

Key metrics:
- boundary_geofence_query_duration_seconds
- boundary_geofence_cache_hit_ratio
- boundary_import_duration_seconds
- boundary_change_rate_per_minute
- fairness_metric_population_density_variance

## Runbooks

See docs/runbooks/boundary-degradation.md for:
- PostGIS replication lag (catchup procedure)
- NLC import failure (manual import from backup)
- Geofence query timeout (index rebuild)
- Fairness metric anomaly investigation

## Degradation contract

Normal: Full read/write with spatial queries and fairness metrics.

Degraded: Read-only from replica; no shapefile import; fairness metrics stale.

Read-only: Serves cached boundary data; no writes; NLC sync paused.

## Chaos scenarios

- postgresql-replica-lag: Delay PostGIS replication 30s+; verify query latency
- geofence-index-corruption: Drop PostGIS spatial index; verify fallback performance
- nlc-import-timeout: Delay shapefile import 10min+; verify graceful degradation
- network-partition: Isolate primary; observe read-only failover

## Statutes

Elections Act §6(1) authority to conduct elections in defined boundaries.
Elections Act §16 delimitation of constituencies and wards per §3 IEBC Act.
Elections Act §29 polling station establishment and location.

## Follow-ups

1. Real-time NLC boundary update subscription (phase 2).
2. Boundary change impact analysis tool for ECC (phase 2).
3. Voter distribution vs fairness anomaly detector (phase 2).
4. Mobile-optimized boundary explorer for field teams.
