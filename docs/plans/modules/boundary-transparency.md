# M18 boundary-transparency

| field | value |
|---|---|
| M-number | M18 |
| Home service | boundary-svc (ADR-0020) |
| Owning work package | WP-2.4 (docs/plans/03-wave-2-trust-core.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M18-boundary-transparency.md |

## Purpose

Publish authoritative electoral boundary data (constituency, ward, polling centre geofence polygons) with cryptographic provenance, track boundary change history, expose fairness metrics derived from KNBS population data, and serve geofence polygons consumed by kit GPS-fence enforcement (M05, M08).

## Surfaces and languages

| surface | technology | path under services/boundary-svc/ |
|---|---|---|
| Public boundary portal | Astro on packages/eel-web-astro | astro/boundary-portal/ |
| Boundary API | Rust (axum + utoipa) | crates/boundary-api/ |
| Batch import + validation | Rust | crates/boundary-batch/ |
| Admin back-office | Django on packages/eel-web-py | django/boundary-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Boundary | boundary_id, type (constituency/ward/polling-centre), code, name, geojson_hash, version, published_at, sig | D | T1-consensus |
| BoundaryChange | change_id, boundary_id, change_type, legal_instrument, effective_date, prior_version | D | T1-audit |
| FairnessMetric | metric_id, election_id, boundary_id, population, registered_voters, ratio, computed_at | D | T2 |
| GeofencePolygon | polygon_id, boundary_id, version, geojson_gz_ref, sig | D | T1-consensus |

## APIs

OpenAPI file: packages/openapi-contracts/boundary/public.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /boundary/v1/boundaries | 1 signed-static | public | n/a | List constituencies/wards; CDN-cached |
| GET /boundary/v1/boundaries/{code} | 1 signed-static | public | n/a | Boundary detail + GeoJSON hash |
| GET /boundary/v1/boundaries/{code}/geojson | 1 signed-static | public | n/a | Signed GeoJSON polygon |
| GET /boundary/v1/changes | 1 signed-static | public | n/a | Change history log |
| GET /boundary/v1/fairness | 1 signed-static | public | n/a | Fairness metrics for current election |
| GET /boundary/v1/geofences | internal gRPC | SVID mTLS | n/a | Kit geofence bundle; consumed by M05/M08 |
| POST /boundary/v1/boundaries | 3 accredited | JWT+SpiceDB | yes | Import new boundary (NLC Gazette) |
| POST /boundary/v1/fairness/compute | 3 accredited | JWT+SpiceDB | yes | Trigger fairness metric computation |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.boundary.boundary_published.v1 | emits | eel.boundary.boundary.v1 | operational | yes (HSM) | exactly-once |
| eel.boundary.boundary_changed.v1 | emits | eel.boundary.boundary.v1 | operational | yes | exactly-once |
| eel.boundary.fairness_computed.v1 | emits | eel.boundary.fairness.v1 | operational | yes | at-least-once idempotent |
| eel.boundary.geofence_updated.v1 | emits | eel.boundary.geofence.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| NLC (National Land Commission) | external | integrations/nlc/ | yes | shapefile import; manual upload fallback |
| KNBS | external | integrations/knbs/ | yes | population data; cached |
| BVR back-end M02 | internal | integrations/bvr-backend/ | yes | registered voter count per boundary |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on boundary publish |
| Transparency log (P05) | internal | integrations/tlog/ | yes | change anchoring queued |
| BVI device M05 | internal | integrations/bvi-device/ | yes | geofence polygon push |
| RTS device M08 | internal | integrations/rts-device/ | yes | geofence polygon push |

## ML and AI

none (fairness metrics are deterministic computations, not ML)

## Dependencies

- Modules: M02 (voter counts), M05 (geofence consumer), M08 (geofence consumer)
- Platform: P05 (audit), P06 (HSM), KNBS data
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro, eel-web-py

## Feature checklist

1. FR-M18-001: import boundary shapefiles from NLC Gazette with legal instrument reference.
2. FR-M18-002: validate topology: no gaps, no overlaps, no slivers; reject invalid shapefiles.
3. FR-M18-003: publish signed GeoJSON polygons for all boundary types.
4. FR-M18-004: HSM-signed boundary publication; verifiable chain from Gazette notice.
5. FR-M18-005: change history: every boundary version retained; diffs computable.
6. FR-M18-006: fairness metrics: population-per-seat, voter-per-seat, Lorenz curve, Gini coefficient per election.
7. FR-M18-007: fairness metric computation triggered on BVR register snapshot.
8. FR-M18-008: geofence polygon bundle for kits: constituency + ward polygons pushed to M05/M08 at kit-prep time.
9. FR-M18-009: public portal: boundary map visualisation + change timeline.
10. FR-M18-010: bulk GeoJSON download for research access.
11. FR-M18-011: shapefile currency tracking: alert IEBC if NLC has published a newer Gazette than imported.
12. FR-M18-012: audit trail: every import, approval, and publish action signed.
13. Boundary publish p95 <= 5 s (after HSM sign).
14. Geofence bundle download <= 30 s.
15. Shapefile currency check runs daily; alert within 24 h of NLC publication.
16. Fairness metrics computed within 4 h of BVR snapshot.

## Acceptance criteria

AC-M18-01: invalid shapefile rejected with topology error list (regression test).
AC-M18-02: boundary publication chain verified from Gazette notice to GeoJSON (integration test).
AC-M18-03: geofence bundle received by M05/M08 kit at prep time (integration test).
AC-M18-04: fairness Gini coefficient matches reference computation on KNBS 2019 data (unit test).
AC-M18-05: shapefile currency alert fires within 24 h of NLC Gazette publication (integration test).
AC-M18-06: 99.5% availability (SLO measurement).

## Applicable edge cases

**data**
EC-DAT-09: Unicode place names (Kikuyu, Luo, Kalenjin scripts); NFC normalisation.
EC-DAT-12: shapefile coordinate system mismatch; reproject to EPSG:4326; reject if outside Kenya bbox.
EC-DAT-15: NLC shapefile format version change; parser version-negotiation; alert.

**security**
EC-SEC-01: boundary signature mismatch; kit rejects geofence polygon.
EC-SEC-03: HSM unavailable; publish blocked; alert.
EC-SEC-06: SSRF via shapefile URL import; allowlist NLC domain only.

**legal**
EC-LEG-07: boundary in court dispute; status flagged on portal; geofence polygon frozen until resolved.

**operations**
EC-OPS-07: NLC API unavailable; manual shapefile upload path; officer upload logged.

**time**
EC-TIM-06: pre-election boundary freeze; admin import blocked after freeze date; court-order override only.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | topology validator, GeoJSON serialiser, fairness metric, shapefile parser, currency checker | nextest | M18-UNIT- |
| property | no boundary gaps or overlaps after import; change history append-only | proptest | M18-PROP- |
| contract | M05 geofence consumer; M08 geofence consumer; M02 voter count (consumer) | Pact | M18-PACT- |
| integration | NLC stub; KNBS stub; SoftHSM publish; tlog anchor | nextest + testcontainers | M18-INT- |
| e2e | import shapefile -> validate -> publish -> geofence push to kit double | Playwright + nextest | M18-E2E- |
| fuzz | shapefile payload; GeoJSON polygon; fairness computation inputs | cargo-fuzz | M18-FUZZ- |
| security | SSRF via shapefile URL; signature bypass; OWASP ZAP | ZAP + Semgrep | M18-SEC- |

## Runbooks and dashboards

- docs/runbooks/boundary-svc-degradation.md (create): NLC unavailable; HSM fail; shapefile topology error; geofence bundle push failure.
- platform/observability/boundary-svc/grafana-boundary.json: shapefile currency lag, publish latency, geofence bundle push success rate.
- platform/observability/boundary-svc/slo-alerts.yaml: 99.5% burn rate; shapefile currency lag > 24 h; boundary publish > 5 s.

## Open questions

- Shapefile currency: NLC does not have a machine-readable publication feed; the current plan polls the NLC Gazette PDF; reliability of PDF parsing for detecting new publications needs validation.
- Boundary dispute handling: the legal process for freezing a disputed boundary polygon during an electoral petition is not specified in regulations; IEBC legal team must confirm the freeze mechanism.
