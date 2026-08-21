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

1. FR-M18-001: import boundary shapefiles from NLC Gazette with legal instrument reference and Gazette number.
2. FR-M18-002: validate topology: no gaps, no overlaps, no slivers; reject invalid shapefiles with structured error list.
3. FR-M18-003: reproject imported coordinates to EPSG:4326; reject any polygon outside the Kenya bounding box.
4. FR-M18-004: publish signed GeoJSON polygons for all boundary types: constituency, ward, polling centre.
5. FR-M18-005: publish signed GeoPackage bundles for every delimitation; retained indefinitely.
6. FR-M18-006: HSM-sign each boundary file with Ed25519; embed public-key fingerprint in file metadata.
7. FR-M18-007: Merkle-inclusion proof for each boundary file within its delimitation bundle.
8. FR-M18-008: signed manifest: Ed25519 over all boundary-file paths and their SHA-256 hashes.
9. FR-M18-009: boundary version history: retain every delimitation revision; diffs computable on demand.
10. FR-M18-010: change-log: machine-readable diff between any two delimitation versions; output signed and published.
11. FR-M18-011: transparency-log anchoring: publish bundle hash to P05 within 60 s of release.
12. FR-M18-012: geofence polygon bundle for kits (constituency + ward); pushed to M05/M08 at kit-prep time via internal gRPC.
13. FR-M18-013: polling-centre lookup API: given GPS coordinates, return polling centre, ward, and constituency.
14. FR-M18-014: candidate constituency check: given candidate address, return elective position and boundary code.
15. FR-M18-015: API search by boundary name, ward code, and GPS coordinates; paginated; rate-limited via token bucket.
16. FR-M18-016: bulk download endpoint: full delimitation package with Merkle inclusion proofs; presigned CDN URL.
17. FR-M18-017: CDN distribution: signed, content-addressed URLs; cache-busting triggered on new boundary version.
18. FR-M18-018: signed URL expiry: bulk-download presigned URL expires after 24 h; no persistent direct-origin links.
19. FR-M18-019: public web portal (Astro, static-first CDN): boundary map visualisation; mobile-first; zero PII.
20. FR-M18-020: interactive map: boundary layer toggle, zoom-to-boundary, and boundary name tooltip.
21. FR-M18-021: WCAG 2.2 AA compliance on web portal verified by automated axe-core scan and manual audit.
22. FR-M18-022: multilingual portal: English and Kiswahili; locale file structure extensible to 11 languages.
23. FR-M18-023: fairness metrics: population-per-seat, voter-per-seat, Lorenz curve, Gini coefficient per election.
24. FR-M18-024: fairness metric computation triggered on BVR register snapshot received from M02.
25. FR-M18-025: shapefile currency check: compare NLC Gazette PDF against imported version daily; alert within 24 h if newer version found.
26. FR-M18-026: pre-election boundary freeze: admin import blocked after configured freeze date; court-order override path retained.
27. FR-M18-027: boundary dispute flag: portal displays dispute notice; geofence polygon frozen until resolution recorded.
28. FR-M18-028: audit trail: every import, approval, and publish action signed and written to immutable audit log.
29. FR-M18-029: admin back-office: NLC shapefile upload, approve, publish workflow; role-based access via SpiceDB.
30. FR-M18-030: SLA enforcement: p95 map and search < 200 ms; 99.9% availability; DDoS-resilient CDN front end.
31. FR-M18-031: rate-limit public API by IP and API key; return 429 with Retry-After header on limit breach.
32. FR-M18-032: OpenAPI contract published at /boundary/v1/openapi.json; contract kept in sync with implementation via CI check.

## Acceptance criteria

AC-M18-01: invalid shapefile rejected with structured topology error list; no partial import persisted (regression test).
AC-M18-02: boundary publication chain verified from Gazette notice through HSM signature to signed GeoJSON (integration test).
AC-M18-03: Merkle inclusion proof for each boundary file validates against the signed manifest (unit test).
AC-M18-04: geofence bundle received by M05/M08 kit double at prep time with matching Ed25519 signature (integration test).
AC-M18-05: fairness Gini coefficient matches reference computation on KNBS 2019 census data within floating-point tolerance (unit test).
AC-M18-06: shapefile currency alert fires within 24 h of simulated NLC Gazette publication (integration test).
AC-M18-07: bulk download package Ed25519 signature verifies end-to-end with bundled public key (e2e test).
AC-M18-08: polling-centre lookup returns correct ward and constituency for 100 sample GPS coordinates across all counties (integration test).
AC-M18-09: portal renders boundary map and passes axe-core WCAG 2.2 AA check with zero violations (e2e Playwright test).
AC-M18-10: English and Kiswahili UI strings fully present; no missing-key fallback text rendered in either locale (e2e test).
AC-M18-11: rate limiter returns HTTP 429 with Retry-After header after burst limit exceeded by 20 % (unit test).
AC-M18-12: pre-election boundary freeze blocks admin import; court-order-less override attempt returns 403 (integration test).
AC-M18-13: p95 map and search endpoint latency stays below 200 ms under 500 concurrent simulated users (load test).
AC-M18-14: transparency log anchor recorded in P05 within 60 s of bundle publication confirmed by tlog query (integration test).

## Applicable edge cases

**data**
EC-DAT-01: malformed boundary payload at import causes parse error; batch rejects the file with a structured error list and leaves prior version untouched.
EC-DAT-02: duplicate boundary code submitted on re-import; system deduplicates by version hash and retains both versions with distinct version IDs.
EC-DAT-09: Unicode place names in Kikuyu, Luo, and Kalenjin scripts; NFC normalisation applied before storage and search indexing.
EC-DAT-12: shapefile coordinate system mismatch detected on ingest; reprojected to EPSG:4326; rejected if centroid falls outside Kenya bounding box.

**identity**
EC-IDN-02: ward name collision across counties; lookup disambiguates by county code prefix returned in response payload.
EC-IDN-10: boundary code reassigned across delimitation cycles; version key in ID isolates the old assignment from the new one.

**security**
EC-SEC-01: boundary Ed25519 signature mismatch on client download; client library rejects the file and logs a tamper alert without caching the bad file.
EC-SEC-03: HSM unavailable at publish time; publish is blocked; incident alert fired; no unsigned boundary output is produced.
EC-SEC-05: JWT replay attack on admin import endpoint; server-side nonce check rejects the duplicate token with 401.
EC-SEC-06: SSRF attempt via shapefile remote URL import field; allowlist restricts outbound fetches to the NLC domain only.
EC-SEC-09: admin uploads a shapefile with an embedded macro payload; binary parser validates MIME type and rejects non-shapefile content.
EC-SEC-12: API key brute-force scan detected; WAF rate-limiter blocks /v1/ burst after 100 req/s per IP for 60 s.
EC-SEC-15: TLS certificate near expiry on CDN origin; automated cert-renewal alert fires 30 days before expiry.
EC-SEC-16: admin session cookie missing HttpOnly and Secure flags; portal sets SameSite=Strict and enforces HSTS on all responses.

**human factors**
EC-HUM-01: first-time visitor has no context for electoral unit types; portal tooltip explains constituency, ward, and polling-centre hierarchy.
EC-HUM-02: map zoomed past maximum boundary detail; portal shows "no boundary detail at this zoom level" guidance inline.
EC-HUM-03: browser Accept-Language header set to Kiswahili; portal auto-selects sw locale without requiring manual toggle.
EC-HUM-05: user enters GPS coordinates in DMS format instead of decimal degrees; portal parser converts and proceeds with lookup.
EC-HUM-08: screen-reader user navigates map; aria-live region announces the focused boundary name on selection.
EC-HUM-11: low-bandwidth connection on first portal load; progressive tile loading with placeholder skeleton avoids blank map.

**scale**
EC-SCL-01: single API pod running; horizontal pod autoscaler adds a replica before p95 latency breaches the 200 ms SLA.
EC-SCL-02: bulk download traffic spike on delimitation release day; presigned CDN URL offloads origin; per-IP rate limit caps burst.
EC-SCL-03: delimitation contains 10 000+ polygons; boundary API streams GeoJSON response without buffering the full payload in memory.
EC-SCL-04: CDN origin-pull storm on new delimitation release; cache warming job pre-populates edge nodes before the public announcement.
EC-SCL-06: read replica lag during high-write boundary import; queries are routed to replica only after replication catch-up is confirmed.
EC-SCL-07: two admin users submit concurrent boundary imports; advisory lock prevents partial-state conflict; second import queued.

**legal**
EC-LEG-02: delimitation legal instrument not yet gazetted; boundary stays in draft status; portal and API do not expose draft to public.

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
