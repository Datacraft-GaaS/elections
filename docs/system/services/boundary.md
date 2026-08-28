# boundary-svc

## 1. Overview

boundary-svc is the authoritative PostGIS store for Kenya's electoral geography — 47
counties, 290 constituencies, 1,450 wards, and ~46k polling stations — plus boundary change
management with public submissions/approvals, geofence point-in-polygon checks over gRPC,
fairness metrics (compactness/population equality), shapefile ingest, and a citizen
boundary explorer Astro site.

- **Runtime**: Rust (axum HTTP + tonic gRPC + background worker crate), one Astro site.
- **Ports**: `HTTP_ADDR` default `0.0.0.0:8080`, `GRPC_ADDR` default `0.0.0.0:9090`
  (services/boundary-svc/crates/boundary-api/src/main.rs:11–16).
- **Entrypoints**: crates/boundary-api/src/main.rs (spawns gRPC service, background
  workers via `boundary_worker::run_all_workers()` :26–28, then serves HTTP).
- Tier T2-routine; advisory T1 for kit geofence lookups; query p95 < 500 ms; geofence
  p99 < 200 ms; RTO 4 h, RPO 30 min (docs/CAPABILITIES.md).

## 2. Ownership & Boundaries

Ground truth: services/boundary-svc/TRACEABILITY.yaml (module M18-boundary-transparency)
and docs/boundary-svc docs directory.

| Requirement | Scope | Evidence crate |
|---|---|---|
| FR-M18-01 | Public read of counties/constituencies/wards/stations | boundary-api handlers |
| FR-M18-02 | Boundary change history for accredited parties | boundary-api::list_boundary_changes |
| FR-M18-03 | Admin snapshot publish (JWT+SpiceDB boundary.admin) | boundary-api::publish_boundary_snapshot |
| FR-M18-04 | Geofence.Check gRPC with signed polygons | boundary-geofence::geo_check, server::GeofenceService |
| FR-M18-05 | Fairness metrics recomputed every 15 min | boundary-worker::fairness |
| FR-M18-06 | Shapefile ingest from NLC/KNBS | boundary-worker::shapefile |

- **Data ownership**: boundaries (versioned geometries), boundary_changes,
  boundary_submissions, fairness_metrics, geofence_polygons, shapefile_imports, outbox,
  audit_log (migrations/V1__init.sql).
- **Upstream**: NLC/KNBS source data via shapefile import. Consumers per docs/EVENTS.md:
  election-day kits use geofence lookups; transparency-svc anchors events.
- Events: published to Kafka topic family `eel.boundary.*` via `eel-outbox-rs` —
  `eel.boundary.boundary_changed.v1`, `eel.boundary.geofence_polygon_signed.v1`
  (docs/EVENTS.md); outbox table exists in the schema.

## 3. Repository Layout

```
services/boundary-svc/
  TRACEABILITY.yaml         # FR/AC matrix with evidence_crates
  crates/
    boundary-api/src/       # main.rs, routes.rs, handlers.rs (~450 lines), openapi.rs
    boundary-domain/        # shared domain types
    boundary-geofence/      # geo_check, tonic GeofenceService, demo_store
    boundary-projections/   # projection helpers
    boundary-worker/src/lib.rs  # fairness, geojson_producer, shapefile workers;
                                # run_all_workers() :21
  proto/geofence.proto      # service Geofence { rpc Check } :9–10
  astro/boundaries/         # citizen explorer site (src/pages/index.astro)
  migrations/V1__init.sql   # PostGIS bootstrap + core tables
  tests/integration_test.rs
  loadtest/, deploy/, docs/ (CAPABILITIES.md, EVENTS.md)
```

## 4. Capabilities & Functions

HTTP routes (crates/boundary-api/src/routes.rs:14–76); handlers in
crates/boundary-api/src/handlers.rs.

### Public reads

| Method | Path | Handler (handlers.rs line) | Purpose |
|---|---|---|---|
| GET | /boundary/counties | `list_counties` :126 | All 47 counties |
| GET | /boundary/constituencies | `list_constituencies` :65 | Constituencies (+GeoJSON) |
| GET | /boundary/constituencies/{code} | `get_constituency` :85 | Single constituency |
| GET | /boundary/wards | `list_wards` :106 | Wards filterable by constituency |
| GET | /boundary/stations | `list_stations` :144 | Stations filterable by ward/const./county |
| GET | /boundary/stations/{code} | `get_station` :166 | Station detail |
| GET | /boundary/changes | `list_boundary_changes` :185 | Change history (JWT required per AC-M18-02-01) |

### WP-2.4 API surface

/api/v1/boundaries list/detail/history (`list_boundaries` :245, `get_boundary` :250,
`get_boundary_history` :255), bulk GeoJSON download (`get_bulk_geojson` :263),
geofence station lookup (:281), simulator compute (`simulator_compute` :288),
fairness/impact metrics (`get_fairness_metrics` :366, `get_impact_metrics` :374),
changes page (:379), submissions (`create_submission` :389).

### Admin

POST /boundary/admin/publish (`publish_boundary_snapshot` :212 — JWT+SpiceDB per doc
comment in routes.rs:9–11), POST /api/v1/changes/{id}/approve (`approve_change` :403),
POST /api/v1/geofence/upload (`upload_geofence` :410 — rejects unsigned polygons, 422,
AC-M18-04-03), POST /api/v1/imports/shapefile (`import_shapefile` :434).

### gRPC

`Geofence.Check(CheckRequest) returns (CheckResponse)` (proto/geofence.proto:10);
served by `GeofenceService` over a polygon store (`demo_store()` wired in main.rs:22).

### Background workers

`run_all_workers()` starts fairness recompute (15-min cadence per AC-M18-05-02),
GeoJSON producer, and shapefile ingest workers (crates/boundary-worker/src/lib.rs:11–21).

### Web

Astro explorer at astro/boundaries/src/pages/index.astro.

## 5. Data Model

migrations/V1__init.sql (PostGIS + uuid-ossp extensions required):

| Table | Purpose | Notable columns/constraints |
|---|---|---|
| boundaries (:11) | Versioned geometries | kind CHECK(county/constituency/ward/polling_station), geom GEOMETRY(4326), effective_from/to, version, status(active/superseded/withdrawn), UNIQUE(code,version), GIST index |
| boundary_changes (:30) | Change ledger | change_type CHECK, old/new_version, gazette_ref, submitted_by, decided_by/at, status |
| boundary_submissions (:47) | Public/party/civil-society submissions | submitter_class CHECK(public/political-party/civil-society), payload_ref |
| fairness_metrics (:57) | Compactness/population-equality/VRA-analog outputs | per-boundary metric rows |
| geofence_polygons (:69) | HSM-signed station polygons | signature required at ingest |
| shapefile_imports (:83) | Ingest job tracking | source refs, status |
| outbox (:98) | Transactional outbox → Kafka eel.boundary.* | per eel-outbox-rs pattern |
| audit_log (:113) | Administrative audit trail | — |

Events: see §2 (EVENTS.md catalogue).

## 6. Operation

- Build/run: `cargo run -p boundary-api`; requires PostgreSQL with PostGIS; env vars
  `HTTP_ADDR`, `GRPC_ADDR`, `RUST_LOG`.
- Test: `cargo test` including tests/integration_test.rs; loadtest/ k6 scenarios.
- Deploy: services/boundary-svc/deploy/service.yaml.
- Degradation: read path is cacheable static-ish geometry; geofence is advisory-T1 — kits
  degrade gracefully when unreachable (documented in docs/CAPABILITIES.md tier note).

## 7. Safeguards & Security

- **Auth posture**: routes.rs documents three tiers — public reads unauthenticated;
  `/boundary/changes` requires JWT (AC-M18-02-01); admin publish/approve/geofence-upload/
  shapefile-import require JWT + SpiceDB `boundary.admin`. The routes.rs doc comment states
  enforcement middleware is "not yet wired here" (routes.rs:9–11) — i.e., authorization is
  specified but not implemented in this router.
- **Integrity**: unsigned geofence polygons rejected with 422 at ingest (AC-M18-04-03);
  HSM countersign flow produces `geofence_polygon_signed.v1` events (docs/EVENTS.md).
  Response headers carry cosign/Merkle headers per AC-M18-01-05.
- **Idempotency**: Idempotency-Key on admin publish prevents duplicate publishes within
  24 h (AC-M18-03-02). Implementation status in handlers not verified line-by-line.
- **Audit logging**: dedicated audit_log table plus decided_by/decided_at on changes.
- **PII handling**: none expected — geographic data only; submissions carry payload refs,
  not identities.
- **Validation**: rich DB CHECK constraints (kind/status/change_type/submitter_class);
  unique(code,version) prevents silent overwrites; effective-dating for gazette timelines.
- **Tests**: tests/integration_test.rs; TRACEABILITY.yaml acceptance criteria map to
  handler-level evidence crates; no auth test suites found.
- **Known deferrals**: JWT/SpiceDB middleware explicitly unwired (routes.rs comment);
  geofence served from a demo store in main.rs:22 (`demo_store()`) rather than the
  database; fairness "every 15 min" cadence asserted in docs but scheduler constants not
  verified here.

## 8. Observability

- Logging/tracing: tracing_subscriber EnvFilter (main.rs:5,:8); startup logs both addrs
  (:18,:39).
- Metrics: none exposed directly; impact/fairness metrics are business APIs, not runtime
  telemetry.
- Health endpoints: no /health route found in routes.rs (not provided); liveness must rely
  on TCP checks or platform probes (gap worth noting).
