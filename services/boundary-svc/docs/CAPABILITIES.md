# boundary-svc capabilities

Authoritative PostGIS store for Kenya's 47 counties, 290 constituencies, 1,450 wards, and 46,229 polling stations, with Polsby-Popper fairness metrics, HSM-signed polygon updates, and a citizen boundary explorer.

**Tier**: T2-routine (boundary queries, explorer, fairness); advisory T1 for geofence lookups consumed by election-day kit configuration.
**SLO**: 99.9% availability. Boundary query p95 < 500ms, p99 < 1s. Geofence point-in-polygon p99 < 200ms.
**RTO**: 4 hours. **RPO**: 30 min.

---

## Contents

1. Purpose and context
2. End-to-end pipelines
3. Domain aggregates and state stores
4. Algorithms and standards
5. Crypto and trust anchoring
6. Event surface
7. API surface
8. Integrations
9. Sharding, indexing, and scale
10. Degradation modes
11. Data lifecycle and retention
12. Observability
13. Testing pyramid
14. Follow-ups and known limitations

---

## 1. Purpose and context

### Modules owned

| Module | Name | RFP ref |
|--------|------|---------|
| M18 | Boundary transparency and geofence | WP-5.1 |

### Timeline role

| Phase | Activity |
|-------|----------|
| T-24 months | Initial shapefile import from NLC; baseline geometries loaded |
| T-12 months | Fairness metrics computed; Polsby-Popper scores published; public portal live |
| T-6 months | Geofence polygons signed by HSM; distributed to bvi-svc for kit config bundles |
| T-3 months | Freeze boundary geometries for election cycle; no further edits without gazette reference |
| Election day | Read-only; geofence queries served from read replicas; no writes permitted |
| T+90 days | Post-election boundary review opens; NLC delta ingestion resumes |

---

## 2. End-to-end pipelines

### 2.1 Shapefile import (NLC/KNBS batch)

```
NLC shapefile (ESRI .shp + .dbf + .prj) -> boundary-worker crate
  -> validate projection (EPSG:4326) -> ST_IsValid geometry check
  -> parse boundary attributes (code, name, administrative_id, population, area)
  -> upsert boundaries table (new version per change)
  -> record boundary_change event -> compute fairness_metrics (Polsby-Popper)
  -> sign updated geofence_polygons (HSM Ed25519) via transparency-svc gRPC
  -> publish outbox -> Kafka: boundary.change.recorded
```

### 2.2 Geofence query (point-in-polygon)

```
client GET /v1/boundaries/station/{station_id}/constituency
  -> boundary-geofence crate
  -> PostGIS: ST_Within(point, geom) on boundaries table (GIST index)
  -> return ContainmentResult { boundary_id, boundary_kind, station_in_boundary: bool }
```

### 2.3 Fairness metric computation

```
boundary-worker (scheduled nightly)
  -> SELECT boundaries WHERE kind='constituency'
  -> For each constituency: area = ST_Area(geom::geography)
  -> perimeter = ST_Perimeter(geom::geography)
  -> polsby_popper = 4 * pi * area / (perimeter^2)  -- range [0,1]; 1 = perfect circle
  -> population_equality = population / avg_population_per_constituency
  -> INSERT fairness_metrics (boundary_kind, metric_name, value, methodology_version)
  -> flag outliers (polsby_popper < 0.1 = highly non-compact)
```

### 2.4 Public explorer (Astro portal)

```
citizen browser -> Astro SSR/static -> GeoJSON boundary tiles
  -> boundary-api: GET /v1/boundaries/{type}/{id} -> GeoJSON Feature
  -> Leaflet.js client-side map rendering
  -> citizen can inspect constituency, ward, station geometry, population, fairness score
```

---

## 3. Domain aggregates and state stores

All tables are in `services/boundary-svc/migrations/V1__init.sql`.

| Table | Purpose |
|-------|---------|
| `boundaries` | Core polygon store. Columns: `id` (UUID), `kind` (county/constituency/ward/polling_station), `code`, `name`, `geom` (GEOMETRY(Geometry,4326)), `effective_from`, `effective_to`, `version`, `status` (active/superseded/withdrawn). GIST index on `geom`. Unique on `(code, version)`. |
| `boundary_changes` | Audit trail for geometry or metadata changes. Columns: `id`, `boundary_id`, `change_type`, `old_version`, `new_version`, `gazette_ref`, `submitted_by`, `decided_by`, `decided_at`, `status` (pending/approved/rejected). |
| `boundary_submissions` | Public and party submissions of proposed boundary changes. Columns: `id`, `submitter_class` (public/political-party/civil-society), `payload_ref`, `submitted_at`, `status`. |
| `fairness_metrics` | Computed compactness and equity metrics. Columns: `id`, `boundary_kind`, `metric_name` (compactness/population-equality/VRA-analog), `value` (DOUBLE PRECISION), `computed_at`, `methodology_version`. |
| `geofence_polygons` | HSM-signed per-station geofence polygons for kit configuration. Columns: `id`, `station_code`, `polygon` (GEOMETRY(Polygon,4326)), `signed_by`, `signature` (BYTEA non-empty), `effective_from`. Unique on `(station_code, effective_from)`. GIST index on `polygon`. |
| `shapefile_imports` | Import audit log. Columns: `id`, `source` (NLC/KNBS), `version`, `file_ref`, `imported_at`, `imported_by`, `verified`. Unique on `file_ref`. |
| `outbox` | Transactional outbox for CloudEvents publication. Columns: `id`, `aggregate_id`, plus standard outbox fields. |

---

## 4. Algorithms and standards

| Capability | Crate/file path | Algorithm | Reference | Standard |
|------------|----------------|-----------|-----------|----------|
| Point-in-polygon geofence | `crates/boundary-geofence` | PostGIS ST_Within using ray-casting over GIST-indexed GEOMETRY | Haines, Eric, "Point in Polygon Strategies", Graphics Gems IV, Academic Press 1994 | OGC Simple Features 1.2.1 |
| Polsby-Popper compactness | `crates/boundary-domain` | PP = 4pi * A / P^2 where A = ST_Area(geom::geography), P = ST_Perimeter(geom::geography) | Polsby and Popper, "The Third Criterion: Compactness as a Procedural Safeguard Against Partisan Gerrymandering", Yale Law and Policy Review 1991 | none binding |
| Population equality metric | `crates/boundary-domain` | ratio = constituency_population / mean_population; outlier flag if ratio > 1.5 or < 0.67 | Baker v. Carr (1962) "one person, one vote" doctrine; Kenya Constitution Art. 89(6) | Kenya Constitution 2010 |
| Shapefile projection check | `crates/boundary-worker` | EPSG:4326 (WGS84 geographic) enforced; reproject via PROJ if needed | EPSG Geodetic Parameter Dataset; proj.org | OGC CRS WKT2, ISO 19111 |
| Geometry validity | `crates/boundary-worker` | ST_IsValid (PostGIS implementation of OGC validity rules: no self-intersections, rings closed) | Ramsey, "PostGIS in Action", Manning 2011 | OGC Simple Features 1.2.1 |
| GeoJSON serialization | `crates/boundary-api` | RFC 7946 GeoJSON (Polygon coordinates in [longitude, latitude] order, EPSG:4326) | Butler, Daly, Doyle, Gillies, Hagen, Schaub, RFC 7946, IETF 2016 | RFC 7946 |
| HSM polygon signing | `crates/boundary-domain` (calls transparency-svc hsm-signer) | Ed25519 over BLAKE3 hash of canonical GeoJSON polygon | Bernstein et al., EUROCRYPT 2011; O'Connor et al., BLAKE3 2020 | ISO/IEC 19790 |

---

## 5. Crypto and trust anchoring

Geofence polygon signing is performed by transparency-svc's HSM signer (key role `sth`). boundary-svc calls the hsm-signer gRPC endpoint to obtain a signature over BLAKE3(canonical_geojson_polygon). The signature and signing key fingerprint are stored in `geofence_polygons.signature` and `geofence_polygons.signed_by`. Kit consumers verify the signature against the transparency-svc public key before accepting a geofence bundle.

Every boundary change is appended to the transparency log as a `custody-events` domain leaf via the `eel.boundary.change.recorded.v1` event, providing an externally verifiable audit trail for all polygon modifications.

---

## 6. Event surface

### Emitted (via transactional outbox, Kafka)

| Event type | Topic | Key fields |
|------------|-------|-----------|
| `eel.boundary.change.recorded.v1` | eel-boundary-events | `boundary_id`, `kind`, `code`, `change_type`, `new_version`, `gazette_ref`, `decided_at` |
| `eel.boundary.snapshot.published.v1` | eel-boundary-events | `snapshot_id`, `kind`, `total_boundaries`, `merkle_root`, `published_at` |
| `eel.boundary.geofence.signed.v1` | eel-boundary-events | `station_code`, `signature_hex`, `key_fingerprint`, `effective_from` |

All envelopes follow CloudEvents 1.0 (CNCF CloudEvents Specification 1.0, 2020). Source: `spiffe://eel.local/svc/boundary-svc`.

### Consumed

boundary-svc has no upstream Kafka consumers. NLC and KNBS data ingested via scheduled batch import jobs in `crates/boundary-worker`. Geofence signatures pulled from transparency-svc gRPC synchronously during import.

---

## 7. API surface

### REST (boundary-api crate, Axum, port 8080)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/boundaries/{type}` | List boundaries by kind (constituency/county/ward/polling_station); paginated |
| GET | `/v1/boundaries/{type}/{id}` | Single boundary geometry and metadata as GeoJSON Feature |
| GET | `/v1/boundaries/station/{station_id}/constituency` | Geofence point-in-polygon: which constituency contains station |
| GET | `/v1/boundaries/fairness-metrics` | Polsby-Popper and population equality per constituency |
| GET | `/v1/boundaries/search` | Full-text search by name (PostgreSQL tsvector) |
| GET | `/v1/boundaries/shapefile/export` | Export all boundaries as GeoJSON FeatureCollection |
| GET | `/v1/boundaries/geofence/{station_code}` | Signed geofence polygon for kit configuration |
| POST | `/v1/boundaries/submissions` | Submit proposed boundary change (public/party/civil-society) |
| GET | `/healthz` | Liveness |
| GET | `/readyz` | Readiness (checks DB, PostGIS extension) |
| GET | `/openapi.json` | OpenAPI 3.1 spec |

### gRPC (proto/boundary.proto, port 8443)

| RPC | Input | Output |
|-----|-------|--------|
| `GetBoundary` | `type: string, id: string` | `Boundary { id, kind, code, name, geojson: string, version }` |
| `QueryGeofence` | `point_lat: double, point_lon: double` | `ContainmentResult { boundary_id, kind, station_in_boundary: bool }` |
| `ListBoundariesByType` | `type: string` | stream of `Boundary` |
| `ExportShapefile` | `type: string` | stream of `ShapefileChunk { chunk: bytes }` |
| `CheckGeofence` | `station_code: string, lat: double, lon: double` | `GeofenceResult { inside: bool, polygon_hash: string, signature_hex: string }` |

The `CheckGeofence` gRPC method is the primary interface consumed by bvi-svc during kit config bundle distribution.

---

## 8. Integrations

| Upstream | Adapter | Method |
|----------|---------|--------|
| NLC shapefiles | `crates/boundary-worker` batch import | Scheduled SFTP/HTTP fetch; ESRI shapefile parser |
| KNBS population data | `crates/boundary-worker` | CSV/API ingest for population-equality metric |
| transparency-svc (polygon signing) | gRPC client in `crates/boundary-domain` | `hsm-signer` gRPC `Sign` RPC |
| transparency-svc (change anchoring) | Kafka outbox relay | Event `eel.boundary.change.recorded.v1` |
| bvi-svc (geofence distribution) | gRPC server (`CheckGeofence`) | Pulled by bvi-svc during kit config bundle generation |

---

## 9. Sharding, indexing, and scale

- PostGIS GIST spatial index on `boundaries.geom` and `geofence_polygons.polygon`: point-in-polygon at 1M queries/day is well within single-node PostGIS capacity.
- Full-text index on `boundaries.name` (GIN, `tsvector`) for search.
- Read replicas serve all public and kit read traffic; writes go only to primary.
- Scale target: 46,229 polling stations; geofence queries peak at poll open (60k kit attestations within 1 hour = 17 queries/s average, 200/s burst). PostGIS GIST handles this with p99 < 200ms.
- Boundary table has at most a few thousand rows; no sharding required.

---

## 10. Degradation modes

| Mode | Trigger | Behavior | Runbook |
|------|---------|----------|---------|
| Normal | All components healthy | Reads and writes at full capacity; fairness metrics computed nightly | docs/runbooks/boundary-svc.md |
| Degraded (NLC import fails) | SFTP/HTTP fetch from NLC unavailable | Existing boundaries served; import retried with backoff; alert fired | docs/runbooks/boundary-svc.md#import-failure |
| Read-only (election day) | Election-day flag set | No writes permitted; all reads from replicas; geofence queries prioritized | docs/runbooks/boundary-svc.md#election-day |
| Read-only (DB primary down) | PG primary failover | Reads from replica continue; writes queued; RTO 4 hours | docs/runbooks/boundary-svc.md#db-failover |

---

## 11. Data lifecycle and retention

| Table | Class (ADR-0016) | Retention |
|-------|-----------------|-----------|
| `boundaries` | Class A: election-critical permanent | Permanent; superseded versions kept with `status=superseded` |
| `boundary_changes` | Class A | Permanent |
| `geofence_polygons` | Class A | Permanent |
| `fairness_metrics` | Class B: operational | 10 years |
| `shapefile_imports` | Class B | 10 years |
| `boundary_submissions` | Class C | 5 years (public submissions) |
| `outbox` | Class C: ephemeral | 30 days |

---

## 12. Observability

### Dashboards (Grafana, d/boundary)

- Boundary query rate (req/s) and latency histogram.
- Geofence query rate and latency (p95, p99 target < 200ms).
- Import job status: last run, rows processed, errors.
- Fairness metric outlier count (constituencies with PP score < 0.1).

### Alerts (Prometheus)

| Alert | Condition | Severity |
|-------|-----------|----------|
| `boundary_geofence_latency_high` | Geofence p99 > 500ms | warning |
| `boundary_import_failed` | Import job error for > 1 hour | warning |
| `boundary_write_during_election` | Write attempt when election-day flag set | critical |
| `boundary_postgis_unavailable` | PostGIS extension check fails | critical |

### Key metrics

- `eel_boundary_geofence_query_duration_seconds` - Histogram: PostGIS point-in-polygon latency.
- `eel_boundary_query_total` - Counter by kind and status.
- `eel_boundary_import_rows_processed` - Gauge: last import size.
- `eel_boundary_fairness_outliers` - Gauge: constituencies failing compactness threshold.

---

## 13. Testing pyramid

| Layer | Tool | Scope |
|-------|------|-------|
| Unit | `cargo test --lib` | Polsby-Popper formula, GeoJSON serialization, polygon hash computation |
| Property | proptest | Any valid WGS84 polygon produces a valid GeoJSON Feature; point-in-polygon consistency |
| Contract | OpenAPI conformance | REST response shapes; GeoJSON Feature schema compliance per RFC 7946 |
| Integration | `cargo nextest run` with testcontainers (PostGIS) | Shapefile import round-trip; geofence query correctness (`cargo test geofence::`) |
| Load | k6 (`loadtest/geofence-query.js --vus 100 --duration 300s`) | 200 concurrent geofence queries; p99 < 200ms |
| Chaos | planned (Wave 3) | PostGIS extension unavailable; NLC SFTP unreachable |
| Formal | none planned | Polygon validity guaranteed by PostGIS ST_IsValid; no TLA+ spec |

---

## 14. Follow-ups and known limitations

- VRA-analog metric is listed as a `metric_name` CHECK constraint value in the migration but computation logic is not yet implemented in `crates/boundary-domain`. Deferred to Wave 2.
- Polsby-Popper computation relies on PostGIS geography type for accurate great-circle area and perimeter on Kenya's coordinate range; verify in staging that EPSG:4326 distances are computed in meters (not degrees) via `::geography` cast.
- Public submission workflow (`boundary_submissions` table) exists in the schema but no API endpoint is wired to a review/approval workflow in `boundary-changes`. Administrative approval via Django admin only for now.
- HSM polygon signing requires transparency-svc to be reachable at import time; if transparency-svc is unavailable, import will fail and retry. This is a hard dependency; add a fallback software-signing mode for boundary-worker if needed.

---

## Appendix A: Polsby-Popper metric interpretation

Polsby-Popper (PP) scores range from 0 (infinitely elongated) to 1 (perfect circle). Kenya's constituencies vary considerably in geography:

| Score range | Interpretation | Example |
|-------------|---------------|---------|
| PP > 0.4 | Compact; broadly acceptable | Urban constituencies |
| 0.1 <= PP <= 0.4 | Moderately elongated; review recommended | Semi-arid corridor constituencies |
| PP < 0.1 | Highly non-compact; potential partisan/geographic issue | Narrow coastal strip constituencies |

Scores are computed using PostGIS geography type (spherical calculations on WGS84 ellipsoid), which gives accurate area and perimeter measurements in square meters and meters respectively, not square degrees. This is critical for Kenya's range of latitudes.

The `fairness_metrics` table stores each score with a `methodology_version` string (e.g., `"PP-v1.0-WGS84"`) so that future methodological changes can be distinguished from score changes.

### VRA-analog metric (planned, Wave 2)

A Voting Rights Act analog metric quantifying the degree to which minority communities (as defined by Kenya's Constitution Chapter 6, special interest groups) are adequately represented in constituency boundaries. Methodology to be defined by the boundary review commission in consultation with civil society. Stored in `fairness_metrics` with `metric_name='VRA-analog'`.

---

## Appendix B: NLC shapefile import workflow in detail

NLC (National Land Commission) provides boundary updates as ESRI shapefiles. The import pipeline in `crates/boundary-worker` handles:

1. **Fetch**: SFTP pull from NLC secure server; SHA-256 verify file integrity against NLC-provided checksum.
2. **Projection check**: Read the `.prj` file; if not EPSG:4326, attempt reprojection via PROJ library (linked via PostGIS). Log reprojection operation in `shapefile_imports`.
3. **Geometry validation**: `ST_IsValid(geom)` for each polygon. Invalid geometries (self-intersections, unclosed rings) are logged and skipped; NLC is notified. A partial import completes with valid records.
4. **Attribute mapping**: ESRI `.dbf` attributes mapped to `boundaries` columns by field name (configurable in `boundary-worker` config TOML). Unknown fields are logged and ignored.
5. **Version assignment**: Each imported boundary gets `version = current_version + 1`. The previous version's `effective_to` is set to `effective_from - 1 day`. Both versions remain in the table with appropriate `status`.
6. **Gazette reference**: All boundary changes must reference a Kenya Gazette notice (`gazette_ref` column in `boundary_changes`). The import tool enforces this; imports without a gazette reference are rejected.
7. **Verification**: `shapefile_imports.verified` flag is set to `true` after a human administrator reviews the import in the Django admin UI. Unverified imports serve read traffic but are flagged in API responses.

---

## Appendix C: Geofence polygon distribution to kits

The `geofence_polygons` table holds per-station polygons signed by transparency-svc's HSM (Ed25519 over BLAKE3 of canonical GeoJSON). These polygons are included in the kit configuration bundles distributed by bvi-svc.

During polling-day, bvi-kit uses the cached geofence polygon to:
1. Verify the kit's GPS location is within the assigned station polygon before allowing identification sessions to begin (time-fence FSM pre-condition).
2. Alert edge-api if the kit's GPS fix places it outside the polygon during an active session (possible kit displacement).

The `effective_from` column ensures the kit uses the correct polygon for the current election cycle. Polygons effective before the current election cycle's lock date are not distributed to kits.

Signature verification on the kit side uses the transparency-svc public key hardcoded in the bvi-kit Flutter app build (pinned at build time; rotated via OTA update only). This prevents a compromised bvi-svc from distributing altered geofence polygons.

---

## Appendix D: PostGIS spatial index performance

The GIST (Generalized Search Tree) index on `boundaries.geom` enables fast bounding-box pre-filtering before exact containment checks. PostGIS geofence query plan for `ST_Within(point, geom)`:

```
Index Scan using boundaries_geom_idx on boundaries
  Index Cond: (geom && point_bbox)
  Filter: ST_Within(point, geom)
```

The bounding-box pre-filter eliminates all non-intersecting polygons in O(log N). The exact containment check (ray-casting) runs only on the few candidates that pass the bounding-box test. For Kenya's 46,229 polling stations each with a geofence polygon, the expected candidate set after bounding-box filter is 1-5 polygons per query, giving p99 < 50ms even without caching.

Cached geofence lookups (Valkey TTL 5 minutes) for repeated queries from the same kit during a session reduce DB load to near zero after the first lookup.

---

## Appendix E: Citizen boundary explorer (Astro portal) design

The public Astro portal (`astro/`) provides citizens with a no-login, read-only view of Kenya's electoral boundaries. Design principles:

- **Static-first**: Astro generates static HTML with island hydration for the interactive map. No server-side rendering on demand; map tiles are pre-generated JSON assets at build time.
- **GeoJSON tiles**: All 47 counties, 290 constituencies, 1,450 wards, and 46,229 station points are pre-exported as GeoJSON at build time. The client loads only the level of detail needed (e.g., constituency polygons on zoom-in).
- **Leaflet.js**: Client-side map rendering with Leaflet.js (BSD-2 license). Boundary polygons loaded as GeoJSON layers. Station points rendered as clustered markers.
- **Fairness score display**: Each constituency popup shows its Polsby-Popper score, population, and registered voter count. Constituencies below 0.1 PP are highlighted in red.
- **No CDN dependencies**: All JavaScript and CSS is bundled at build time; no external CDN calls at runtime (CSP enforced).
- **Accessibility**: ARIA labels on map regions; screen-reader-accessible constituency data table as an alternative to the visual map.
- **Update cadence**: Portal rebuilt and redeployed whenever a boundary change is approved (triggered by `boundary.change.recorded` CloudEvent consumed by a build pipeline).

---

## Appendix F: Boundary change governance

All boundary changes follow a multi-step governance workflow before they are applied to the `boundaries` table:

1. **Submission**: NLC submits a shapefile with a Gazette reference number via the Django admin UI. The file is stored in MinIO and a `boundary_submissions` row created.
2. **Technical validation**: boundary-worker runs ST_IsValid, projection check, and attribute mapping. Results reported in the Django admin UI.
3. **Legal verification**: Commission legal team verifies the Gazette reference is valid and matches the submitted geometry. The `boundary_changes.decided_by` and `decided_at` fields are set.
4. **Approval**: Senior Commission official sets `boundary_changes.status = 'approved'` via Django admin. This triggers the import pipeline to apply the change.
5. **Transparency log anchor**: The approved change hash is appended to the transparency log as a `custody-events` leaf within 1 minute of approval.
6. **Portal rebuild**: The Astro portal is rebuilt and redeployed within 15 minutes of anchoring.
7. **Rejection**: If the change is rejected, `boundary_changes.status = 'rejected'` is set; the original boundary geometry is preserved unchanged; the rejection reason is logged.

This workflow ensures no boundary change can be applied without a valid Gazette reference and dual authorization (technical + legal), providing an audit trail that satisfies the Independent Electoral and Boundaries Commission Act requirements.

---

## Appendix G: boundary-projections and boundary-domain crate roles

The `crates/boundary-projections` crate maintains read-model projections derived from boundary change events. This separates write-side concerns (geometry storage, change governance) from read-side concerns (API serving, caching):

- **Write side** (`boundary-domain`): Handles boundary aggregate state, version management, fairness metric computation, and geofence signing.
- **Read side** (`boundary-projections`): Maintains denormalized read models for API serving. Key projections: boundary search index (PostgreSQL GIN tsvector on name + code), constituency-to-stations mapping (used by rts-svc for tally routing), county-to-constituency hierarchy (used by portal for drill-down navigation).
- **boundary-api** crate: REST and gRPC API handlers that query the read-model projections and `boundaries` table. Stateless; scales horizontally; no write access to core tables.
- **boundary-worker** crate: Background worker for shapefile import, KNBS population sync, and fairness metric computation. Runs as a separate deployment (Kubernetes CronJob for scheduled imports; event-driven for change-triggered metric recomputation).

This separation allows boundary-api to be scaled out to handle read traffic without affecting the write-side import pipeline, and allows boundary-worker to run heavy shapefile import operations without impacting API response times.


---

## Appendix H: boundary-svc data dictionary alignment

The `data_dictionary.yaml` file at the service root defines canonical names and types for all data elements. Key mappings to the PostgreSQL schema:

| YAML element | PG column | Type | Notes |
|-------------|-----------|------|-------|
| `boundary.id` | `boundaries.id` | UUID | Generated by gen_random_uuid() |
| `boundary.kind` | `boundaries.kind` | TEXT CHECK | county/constituency/ward/polling_station |
| `boundary.geom` | `boundaries.geom` | GEOMETRY(Geometry,4326) | WGS84; GIST indexed |
| `boundary.version` | `boundaries.version` | INT | Increments on every approved change |
| `fairness.polsby_popper` | `fairness_metrics.value WHERE metric_name='compactness'` | DOUBLE PRECISION | PP score [0,1] |
| `geofence.signature` | `geofence_polygons.signature` | BYTEA | Ed25519 over BLAKE3(canonical GeoJSON) |

The data dictionary is authoritative for external API consumers; this CAPABILITIES.md is authoritative for internal engineering.
