# Boundary transparency (M18)

Module M18 (`boundary-transparency`) owned by `boundary-svc`. Publishes authoritative electoral boundary data (constituency, ward, polling-centre polygons) with cryptographic provenance, tracks change history, exposes fairness metrics derived from KNBS population data, and serves signed geofence polygons consumed by kit GPS-fence enforcement (M05, M08).

## Contents

1. What boundary transparency does (vs M05 kit GPS-fence enforcement)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What boundary transparency does (vs M05 kit GPS-fence enforcement)

- **M05** = enforces geofence polygons on the kit at voter identification time; rejects authentication attempts outside the station boundary.
- **M18** = the canonical source of those polygons: ingests shapefiles from NLC, validates topology, HSM-signs each boundary file, publishes GeoJSON bundles, computes fairness metrics, and pushes geofence bundles to M05/M08 at kit-prep time.

M18 is read-only for the public; only accredited NLC-shapefile importers may write.

## 2. Data plane

Four aggregates (per `services/boundary-svc/crates/boundary-api/src/models.rs` and the M18 spec):

| Aggregate | Purpose | Data class |
|---|---|---|
| **Boundary** | boundary_id, type (constituency/ward/polling-centre), code, name, geojson_hash, version, published_at, sig | D |
| **BoundaryChange** | change_id, boundary_id, change_type, legal_instrument, effective_date, prior_version | D |
| **FairnessMetric** | metric_id, election_id, boundary_id, population, registered_voters, ratio, Polsby-Popper score, Gini coefficient, computed_at | D |
| **GeofencePolygon** | polygon_id, boundary_id, version, geojson_gz_ref, sig | D |

Migrations land under `services/boundary-svc/migrations/`. PostGIS extension required for spatial queries.

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **NLC Gazette shapefile** | Per-delimitation (irregular) | `integrations/nlc/` | Manual upload via Django admin | Topology validator rejects partial import; no partial state persists |
| **KNBS census data** | Per-census update | `integrations/knbs/` | SFTP + PGP | Cached; fairness computation skipped until fresh data arrives |
| **BVR back-end M02** | On-demand | `integrations/bvr-backend/` | SVID mTLS | Circuit breaker; voter counts cached |
| **HSM P06** | Per publish | `integrations/hsm/` | SoftHSM in dev | Publish blocked until HSM signs |

Every adapter ships a `double/` subdirectory so the ingest pipeline runs end-to-end before live NLC endpoints are provisioned.

## 4. Pipeline

Files under `services/boundary-svc/crates/boundary-batch/src/`:

```
NLC shapefile upload (Django admin)
      -> topology validator
            (no gaps, no overlaps, no slivers; structured error list on fail)
      -> EPSG reprojection to EPSG:4326
            (reject if centroid outside Kenya bounding box via PostGIS ST_Within)
      -> Geofence.Check point-in-polygon via PostGIS ray-casting
            (validates GPS coordinates for polling-centre lookup)
      -> Polsby-Popper compactness score computed per boundary
            (Polsby and Popper 1991, J. Law Review 9:4)
      -> fairness metrics: population-per-seat, voter-per-seat,
            Lorenz curve, Gini coefficient, triggered by M02 register snapshot
      -> HSM Ed25519 sign per boundary file (P06)
      -> Merkle inclusion proof within delimitation bundle
      -> signed manifest: Ed25519 over all boundary-file paths + SHA-256 hashes
      -> publish GeoJSON bundle to CDN; signed URL expires 24 h
      -> emit CloudEvents via eel-outbox-rs:
            eel.boundary.boundary_published.v1
            eel.boundary.boundary_changed.v1
            eel.boundary.fairness_computed.v1
      -> anchor bundle hash in P05 transparency log within 60 s
      -> push geofence bundle to M05 BVI kits + M08 RTS kits via internal gRPC:
            eel.boundary.geofence_updated.v1
      -> daily NLC currency check: compare Gazette PDF against imported version;
            alert within 24 h if newer version found
```

Every step goes through the transactional outbox (`packages/eel-outbox-rs`). Debezium picks up outbox rows and publishes to `eel.boundary.boundary.v1` and `eel.boundary.fairness.v1`.

## 5. API surface

Contract: `packages/openapi-contracts/boundary/public.yaml`. Write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /boundary/v1/boundaries` | Public | List constituencies/wards; CDN-cached |
| `GET /boundary/v1/boundaries/{code}` | Public | Boundary detail + GeoJSON hash |
| `GET /boundary/v1/boundaries/{code}/geojson` | Public | Signed GeoJSON polygon |
| `GET /boundary/v1/changes` | Public | Change history log |
| `GET /boundary/v1/fairness` | Public | Fairness metrics for current election |
| `GET /boundary/v1/geofences` | Internal gRPC (SVID mTLS) | Kit geofence bundle; M05/M08 at prep time |
| `GET /boundary/v1/boundaries/{code}/proof` | Court registrar | Merkle inclusion proof |
| `POST /boundary/v1/boundaries` | NLC importer (JWT + SpiceDB) | Import boundary from Gazette (idempotent) |
| `POST /boundary/v1/fairness/compute` | Accredited admin | Trigger fairness metric computation |

Every write is HSM-signed via the response-signing middleware in `packages/eel-api-server-rs`. All reads and writes are audited into `eel_boundary_audit_shadow`.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Public boundary portal (Astro) | `services/boundary-svc/astro/boundary-portal/` | Interactive map, boundary layer toggle, name search, WCAG 2.2 AA |
| Admin back-office (Django) | `services/boundary-svc/django/boundary-admin/` | NLC shapefile upload, approve, publish; role-gated via SpiceDB |
| Boundary API (Rust axum) | `services/boundary-svc/crates/boundary-api/` | REST + gRPC surface for internal and external consumers |

Languages: EN + SW for all public surfaces; extensible to additional Kenyan languages. WCAG 2.2 AA verified via automated axe-core scan and manual audit.

## 7. Integrity + anti-tamper

- **PostGIS ray-casting** -- every Geofence.Check is a PostGIS `ST_Contains` point-in-polygon against EPSG:4326 polygons; coordinates outside the Kenya bounding box are rejected at ingest.
- **Polsby-Popper scores** -- compactness ratio `(4 * pi * area) / perimeter^2` published per boundary per delimitation (Polsby and Popper 1991); complements Gini coefficient and Lorenz curve for public fairness review.
- **Ed25519 HSM signatures** -- each boundary file signed by Commissioner KEK (per-cycle from `eel_kek_generations`); public-key fingerprint embedded in file metadata.
- **Merkle inclusion proofs** -- every boundary file is a leaf in the delimitation bundle; proof verifiable by any party holding the signed manifest.
- **Signed manifest** -- Ed25519 over all boundary-file paths and their SHA-256 hashes; published alongside the bundle.
- **P05 anchoring** -- bundle hash anchored in the transparency log within 60 s of release; `GET /boundary/v1/boundaries/{code}/proof` returns the inclusion proof.
- **Geofence push integrity** -- geofence bundles delivered to M05/M08 via internal mTLS gRPC; kit verifies Ed25519 signature before accepting; unsigned bundles refused.
- **Currency alert** -- daily NLC PDF parse detects newer Gazette version; alert fires within 24 h.
- **Boundary freeze** -- admin import blocked after configured election freeze date; court-order override path retained but requires HSM signature.

## 8. Kenya statute alignment

- **Elections Act s.26 (constituency delimitation)** -- boundary data sourced from NLC legal instruments; every boundary record includes the Gazette number and effective date.
- **National Land Commission Act s.15 (delimitation functions)** -- NLC is the authoritative source; M18 ingests and publishes but does not define boundaries.
- **Kenya Data Protection Act s.26 (minimisation)** -- no voter-identifiable data in any boundary or fairness record; KNBS population aggregates only.
- **Elections (Technology) Regulations 2017** -- signed bundles + Merkle anchoring + KEBS-certifier attestation before each delimitation publish.

## 9. Honest caveats + external prerequisites

- NLC does not have a machine-readable publication feed. The daily currency check parses the NLC Gazette PDF; reliability depends on PDF structure stability. Datacraft must validate PDF parsing against at least three prior Gazette editions before go-live.
- The KNBS data-sharing agreement must be formalised before fairness metrics reflect current census data; the adapter and double are ready but the live SFTP endpoint is not provisioned.
- Boundary dispute handling: the legal process for freezing a disputed polygon during an electoral petition is not specified in the regulations. IEBC legal must confirm the freeze mechanism before the dispute-flag feature is enabled.
- The Polsby-Popper score is published as an informational metric. It has no statutory weight in Kenyan delimitation law; its publication is advisory and must not be presented as a binding fairness criterion.
- CDN pre-warming on delimitation release day requires a separate operational step before the public announcement; the boundary team must coordinate with the CDN operator.

## 10. Test coverage + operational runbooks

Coverage under `services/boundary-svc/crates/boundary-api/tests/` and `crates/boundary-batch/tests/`:
- Unit: topology validator, GeoJSON serialiser, Polsby-Popper computation, Gini, shapefile parser, currency checker.
- Property (`proptest`): no boundary gaps or overlaps after import; change history append-only.
- Contract (Pact): M05 geofence consumer; M08 geofence consumer; M02 voter-count consumer.
- Integration (`testcontainers`): NLC stub; KNBS stub; SoftHSM publish; P05 tlog anchor.
- End-to-end (Playwright + nextest): import shapefile, validate, publish, geofence push to kit double.
- Load: p95 map and search endpoint < 200 ms under 500 concurrent users.
- Security: SSRF via shapefile URL; signature bypass; OWASP ZAP scan.

Runbooks under `docs/runbooks/`:
- `boundary-svc-degradation.md` -- NLC unavailable; HSM fail; shapefile topology error; geofence bundle push failure.
- `election-day-mode-toggle.md` -- boundary ingest freeze window T-30 through election day.
- `witness-divergence-response.md` -- if P05 transparency-log witnesses diverge on a boundary bundle.

Related: full spec at `docs/plans/modules/boundary-transparency.md`; capability catalogue at `services/boundary-svc/docs/CAPABILITIES.md`.
