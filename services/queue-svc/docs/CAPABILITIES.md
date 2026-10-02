# queue-svc capabilities and algorithms

Comprehensive capability catalogue for the queue transparency service. Every capability lists
the algorithm, source reference (paper, statutory authority, or standard), and the crate
or component that implements it.

## Contents

1. Modules and capabilities overview
2. End-to-end data flows
3. Snapshot fusion algorithm (M19)
4. Wait-time percentile computation (M19)
5. Disenfranchisement flag heuristic (M19)
6. S.14A recommendation and Chair HSM signature (M19)
7. Computer vision head-count pipeline (M19)
8. Real-time station map (M19)
9. Anti-gaming and signal integrity controls
10. Operational excellence
11. Standards and interoperability
12. Validation harnesses
13. Data lifecycle and security
14. API surface
15. Events
16. Summary

---

## 1. Modules and capabilities overview

| Module | Slug | Size | Primary crate | Notable risk |
|---|---|---|---|---|
| M19 | queue-transparency | M | crates/queue-pipeline | signal fusion quality; CV head-count accuracy |

Tier: T2. Availability SLO: 99.9%. Queue snapshot latency p95 < 2 s. Station map update latency < 30 s.

RTO: 1 hour. RPO: 5 min.

Statutory authority: Kenya Elections Act §14A (extension of polling hours from 17:00 to 19:00 when disenfranchisement risk is established).

---

## 2. End-to-end data flows

### 2.1 Snapshot ingestion and fusion

```
biometric-edge.identification.submitted
  -> Kafka consumer (at-least-once idempotent)
  -> queue-pipeline: extract kit_throughput (voters_served delta, clock delta)
  -> merge with clerk_reports (field-mobile.queue.reported)
  -> merge with cv_headcount (py/queue_cv/pipeline.py via ONNX)
  -> merge with qr_scan_count (biometric-edge.qr.scanned)
  -> QueueSnapshot upserted (30 s interval per station)
  -> Elixir LiveView PubSub: eel:queue_map broadcast
```

### 2.2 Disenfranchisement detection

```
QueueSnapshot with queue_length and closure_time_estimate
  -> DisenfranchisementHeuristic: P(voter reaches desk before closure) < threshold
  -> DisenfranchisementFlag created
  -> eel.queue.disenfranchisement_flagged.v1 emitted to ecc-svc
  -> If P < critical_threshold AND time_to_closure < 90 min:
       -> S14ARecommendation created (Chair HSM signature attached)
       -> eel.queue.s14a_recommended.v1 emitted
```

---

## 3. Snapshot fusion algorithm (M19)

### 3.1 Multi-source weighted median

Crate: `crates/queue-pipeline/src/fusion.rs`.

Four independent signal sources fused into one `QueueSnapshot` per station per 30 s window:

| Source | Signal | Default weight |
|---|---|---|
| Kit throughput (BVI identification events) | voters_served_delta / time_delta -> throughput_rate | 0.45 |
| Clerk reports (field-mobile.queue.reported) | reported_queue_length | 0.30 |
| CV head-count (py/queue_cv/pipeline.py) | cv_head_count, cv_confidence | 0.15 |
| QR scan events (biometric-edge.qr.scanned) | qr_scan_count_delta | 0.10 |

Fusion method: weighted median (Edgeworth 1888) across present sources. Missing source weight redistributed proportionally. Confidence score = weighted sum of present source weights. Snapshot rejected if confidence_score < 0.25 (fewer than 1 effective source).

### 3.2 Throughput-based queue length estimation

When direct queue length is unavailable, inferred from throughput rate and arrival rate:

```
queue_length_estimated = max(0, arrival_rate_est - throughput_rate) * elapsed_seconds
```

`arrival_rate_est` from exponential moving average of prior 3 windows (alpha = 0.3). Capped at station-registered capacity from boundary-svc.

### 3.3 Schema (migrations/V001__queue_snapshots.sql, V002__queue_events.sql)

`eel_queue_snapshots`: snapshot_id, station_id, captured_at, queue_length, voters_served, avg_wait_seconds, confidence_score, fusion_sources JSONB.
`eel_queue_events`: event_id, station_id, event_type (snapshot/anomaly/flag/s14a), payload JSONB, recorded_at.

---

## 4. Wait-time percentile computation (M19)

### 4.1 Algorithm

Crate: `crates/queue-pipeline/src/wait_time.rs`.

Per-station wait time estimated from queue length and throughput rate:

```
wait_time_seconds = queue_length / throughput_rate
```

Population-level percentiles computed over all active stations using t-digest (Dunning and Ertl 2019, Computing Accurate Quantiles Using T-Digest) with compression factor 100. Percentiles p50, p75, p90, p95 published per snapshot cycle.

Station-level wait time bounded: `min(wait_time_seconds, seconds_until_poll_close)`. Negative values clamped to 0.

### 4.2 Closure time estimate

```
closure_time_estimate = poll_close_time - now()
voters_remaining = queue_length
throughput_needed = voters_remaining / closure_time_estimate
risk = throughput_needed / current_throughput_rate
```

`risk > 1.0`: voters cannot all be served before poll close at current throughput (disenfranchisement risk).

| Algorithm | Reference |
|---|---|
| Weighted median fusion | Edgeworth 1888; Mosteller and Tukey 1977 Statistical Data Analysis |
| t-digest percentile | Dunning and Ertl 2019; github.com/tdunning/t-digest |
| Exponential moving average | Hunter 1986 Journal of Quality Technology |

---

## 5. Disenfranchisement flag heuristic (M19)

### 5.1 Flag criteria

Crate: `crates/queue-pipeline/src/disenfranchisement.rs`.

A `DisenfranchisementFlag` is created when ALL of the following hold:

1. `closure_risk > 0.8` (voters remaining / throughput rate > 80 % of remaining poll time).
2. `queue_length > 50` (absolute minimum to distinguish noise from real queues).
3. `confidence_score >= 0.5` (at least two independent sources confirm the signal).
4. `time_to_poll_close <= 120 min` (S.14A can only be invoked within 2 h of scheduled close).

### 5.2 Cross-source validation

Before flagging, the heuristic runs a cross-source consistency check:

- Kit throughput estimate and CV head-count must agree within 40 % relative error.
- If CV confidence < 0.6, CV source excluded from cross-check (degraded mode).
- Disagreement beyond tolerance: `DisenfranchisementFlag.quality = UNCERTAIN`; ECC alerted to manually verify.

---

## 6. S.14A recommendation and Chair HSM signature (M19)

### 6.1 Recommendation criteria

Crate: `crates/queue-pipeline/src/s14a.rs`.

`S14ARecommendation` generated when:
- `DisenfranchisementFlag.quality IN (CONFIRMED, UNCERTAIN)` for a station.
- `closure_risk > 0.9` (strong signal).
- `time_to_poll_close BETWEEN 30 min AND 120 min` (within actionable window per §14A).

Recommendation payload: `{ election_id, station_id, flag_id, estimated_voters_at_risk, recommended_extension_minutes, generated_at, justification }`.

### 6.2 Chair HSM signature

Signing: PKCS#11 HSM, key role `s14a_recommender` (Ed25519). Signature over `SHA-256(recommendation_payload_cbor)`. Stored in `eel_s14a_recommendations`: recommendation_id, station_id, recommendation_time, payload JSONB, hsm_signature, chair_key_id.

The HSM signature authenticates the automated recommendation as originating from the queue-svc production instance; human Chair action (acceptance or rejection) is separately recorded in ecc-svc via the S.44A dual-control state machine.

| Algorithm | Reference |
|---|---|
| Ed25519 HSM signing | Bernstein et al. IACR ePrint 2011 |
| PKCS#11 interface | OASIS PKCS#11 v3.0 2020 |
| SHA-256 payload hash | FIPS 180-4, NIST 2015 |
| CBOR serialization | RFC 7049 (Bormann, Hoffman 2013) |

---

## 7. Computer vision head-count pipeline (M19)

### 7.1 Model

Python worker: `py/queue_cv/pipeline.py`. ONNX Runtime inference.

- Base model: CSRNet (Li et al. CVPR 2018) density-map crowd counting, adapted for polling station crowd geometry.
- Alternate model: MCNN (Zhang et al. CVPR 2016) for low-resolution CCTV feeds.
- Input: JPEG frame from CCTV feed (where available); 640x480 minimum resolution.
- Output: `{ head_count: int, density_map: ndarray, confidence: float }`.
- Confidence calibrated via temperature scaling (Guo et al. ICML 2017).
- Inference latency p95 < 1 s per frame (int8 quantized ONNX model).

| Algorithm | Reference |
|---|---|
| CSRNet crowd counting | Li, Zhang, Chen CVPR 2018 |
| MCNN crowd counting | Zhang, Zhou, Li, Shang CVPR 2016 |
| Temperature scaling calibration | Guo, Pleiss, Sun, Weinberger ICML 2017 |
| ONNX Runtime int8 quantization | Bhandare et al. IEEE 2019 |

### 7.2 CCTV feed integration

CV pipeline polls CCTV RTSP feeds (where available at station). On feed unavailability, CV source excluded and weight redistributed. `cv_estimates` table: station_id, timestamp, head_count, confidence_score, model_version, frame_resolution.

---

## 8. Real-time station map (M19)

### 8.1 Elixir LiveView map

Component: `ex/queue_live/` (Elixir Phoenix LiveView). Public access (no authentication).

- PubSub topic `eel:queue_map`: snapshot_id, station_id, queue_length, wait_time_p50, closure_risk, disenfranchisement_flagged. No PII.
- Map renders all ~47,000 stations; colour-coded by closure_risk (green < 0.5, amber 0.5-0.8, red > 0.8).
- Update cadence: 30 s (station snapshot interval). Burst mode: 10 s cadence when S.14A candidates > 0.
- Geographic coordinates sourced from boundary-svc PostGIS polygons.

### 8.2 Public API

`GET /v1/queue/stations`: GeoJSON FeatureCollection with per-station queue properties. Used by public third-party integrators and media.

### 8.3 Constituency and county roll-up

`GET /v1/queue/constituencies/{constituency_id}/summary`: aggregated queue_length (sum), max closure_risk, and disenfranchisement_count across all stations in the constituency. Computed from `eel_queue_snapshots` via pre-aggregated `eel_queue_constituency_summary` (upserted every 30 s by background worker). Prevents N+1 map-tile queries from public clients.

`GET /v1/queue/counties/{county_id}/summary`: county-level roll-up (47 counties). Used by ECC county-level dashboard tiles.

### 8.4 S.14A candidate map layer

When `eel_s14a_recommendations` contains active (unresolved) recommendations, the station map activates a dedicated S.14A overlay layer: affected stations rendered with amber pulsing marker distinct from the standard closure_risk colour coding. Layer persists until recommendation is accepted or rejected by ecc-svc S.44A dual-control. Overlay data served from `GET /v1/queue/s14a-candidates` (GeoJSON format).

---

## 9. Anti-gaming and signal integrity controls

### 9.0 QR scan deduplication

Crate: `crates/queue-pipeline/src/qr_dedup.rs`.

QR scan events carry `{ scan_id: UUID7, station_id, scanned_at, scanner_device_id }`. Deduplication fingerprint: `blake3(scan_id || station_id || 5-min-bucket)`. Fingerprint stored in `eel_qr_scan_fingerprints` (INSERT OR IGNORE; ttl-pruned at 24 h). Duplicate scans from the same device within the same 5-minute window are dropped; the first scan is counted. This prevents inflated queue length estimates from polling staff refreshing a QR scan page.

| Control | Mechanism | Crate / component |
|---|---|---|
| Signal source authentication | Kit telemetry authenticated via SPIRE SVID + attestation; unsigned signals rejected | crates/queue-pipeline |
| Cross-source consistency check | 40 % relative-error tolerance between kit throughput and CV head-count | crates/queue-pipeline/src/disenfranchisement.rs |
| Confidence floor | Snapshots with confidence < 0.25 discarded; not used for S.14A recommendations | crates/queue-pipeline/src/fusion.rs |
| Clerk report rate limit | Max 10 clerk queue reports per station per 30 min; excess discarded | crates/queue-pipeline |
| QR scan deduplication | QR scan events deduplicated on (station_id, scan_id, 5-min bucket) | crates/queue-pipeline |
| HSM-signed recommendations | S.14A recommendations carry HSM signature; unsigned recommendations rejected by ecc-svc | crates/queue-pipeline/src/s14a.rs |

---

## 10. Operational excellence

| Capability | Mechanism |
|---|---|
| Kit signal loss | Prometheus alert when station offline > 5 min; fallback to clerk + CV sources |
| CV pipeline timeout | Model reload triggered; queue snapshot uses kit + clerk sources only (degraded) |
| Queue fusion lag | Consumer scaling via HPA when Kafka consumer lag > 10,000 messages |
| S.14A recommendation spike | ECC notified immediately via eel.queue.s14a_recommended.v1; duty officer paged |
| Station map cache | Last snapshot served from Valkey on Postgres unavailability (max staleness 5 min) |
| k6 load test | `loadtest/queue_load.js` targets 500 VUs / 300 s; 30 s snapshot intervals across 10k stations |
| HPA | CPU 70 %, memory 80 %; min 2, max 16 replicas |
| Poll-close burst | At T-90 min before poll close: snapshot cadence increases from 30 s to 10 s; queue-live dashboard enters burst mode; ECC ECC and IAB dashboards prioritise disenfranchisement tiles |
| Degraded confidence fallback | When confidence_score < 0.5: snapshot published with quality=DEGRADED label; public map shows amber confidence indicator; S.14A not generated from degraded snapshots alone |
| CV feed health | RTSP feed liveness checked every 60 s per station; offline feed excluded from fusion; `eel_cv_feed_health` table updated; Prometheus alert when > 20 % of stations with CCTV have feed offline |

### 10.1 S.14A operational window

The S.14A mechanism operates within a legally bounded window per Kenya Elections Act §14A:

- Earliest actionable time: T-120 min before scheduled poll close (17:00) = 15:00.
- Latest actionable time: T-30 min = 16:30 (recommendations generated after 16:30 are advisory only; queue-svc labels them advisory_only=true).
- queue-svc enforces the window at recommendation generation time. Recommendations outside the window are logged but not emitted as `eel.queue.s14a_recommended.v1` events; they are available via `GET /v1/queue/s14a-candidates?include_advisory=true` for audit purposes.
- Extension duration: fixed 2 h (17:00 to 19:00) per statute; queue-svc recommendation payload always sets `recommended_extension_minutes=120` regardless of computed closure_risk magnitude.

---

## 11. Standards and interoperability

| Standard | Coverage | Crate / component |
|---|---|---|
| Kenya Elections Act §14A | S.14A extension authority; recommendation criteria | crates/queue-pipeline/src/s14a.rs |
| PKCS#11 v3.0 | HSM interface for S.14A recommendation signing | crates/queue-pipeline/src/s14a.rs |
| RFC 7049 CBOR | Recommendation payload serialization before HSM signing | crates/queue-pipeline |
| CloudEvents 1.0 | All consumed and emitted event envelopes | crates/queue-pipeline |
| GeoJSON (RFC 7946) | Station map API response format | crates/queue-pipeline/src/api.rs |
| OpenAPI 3.1 | API contract at packages/openapi-contracts/queue/ | crates/queue-pipeline |

---

## 12. Validation harnesses

| Harness | Scope | Command |
|---|---|---|
| Fusion unit tests | Weighted median, missing source redistribution | `cargo test fusion:: --lib` |
| S.14A recommendation tests | Criteria coverage, HSM signature | `cargo test s14a_recommendation:: --lib` |
| Disenfranchisement heuristic tests | Boundary conditions, cross-source consistency | `cargo test disenfranchisement:: --lib` |
| CV pipeline accuracy | CSRNet density map vs ground truth head count | `cd py && uv run pytest tests/test_cv_accuracy.py` |
| t-digest percentile tests | Accuracy at extreme quantiles | `cargo test wait_time:: --lib` |
| Integration tests | Full snapshot pipeline with mock Kafka | `cargo nextest run` |
| k6 load test | 500 VUs; 30 s snapshots; 10k stations | `cd loadtest && k6 run queue_load.js` |

---

## 13. Data lifecycle and security

### 13.1 Encryption

Snapshot data: no PII; stored in plaintext. S.14A recommendations: HSM-signed payload in `eel_s14a_recommendations.payload` (CBOR; plaintext station data only; no voter PII). CV density maps: not persisted (inference output discarded after head_count extraction; only the scalar count and confidence stored). CCTV RTSP stream credentials: stored in OpenBao (HashiCorp Vault-compatible) at the ml/vault server (192.0.2.11 per SERVICE_CATALOGUE.md); retrieved at CV pipeline startup via SPIRE SVID-authenticated token.

### 13.2 Audit

`eel_queue_events` is an append-only event log (INSERT only; no UPDATE or DELETE trigger). Every S.14A recommendation retained permanently. `eel_disenfranchisement_flags` retained 7 years post-election per Elections Act §39(1C).

### 13.3 Retention and partitioning

- `eel_queue_snapshots`: 90-day rolling partition (time-series; election-night data retained 3 years). Partitioned by `captured_at` (RANGE, monthly). Partition pruning active for all dashboard queries via `captured_at >= now() - interval '30 min'` filter.
- `eel_disenfranchisement_flags`: 7 years post-election.
- `eel_s14a_recommendations`: permanent.
- `eel_cv_estimates`: 90 days (operational monitoring only).
- `eel_qr_scan_fingerprints`: 24-hour TTL; nightly prune job removes rows where `recorded_at < now() - interval '24 hours'`.
- `eel_queue_constituency_summary`: updated in-place (upsert); no historical retention.
- `eel_queue_events`: permanent (audit log).

### 13.2 Audit

`eel_queue_events` is an append-only event log (INSERT only; no UPDATE or DELETE trigger). Every S.14A recommendation retained permanently. `eel_disenfranchisement_flags` retained 7 years post-election per Elections Act §39(1C).

### 13.3 Retention

- `eel_queue_snapshots`: 90-day rolling partition (time-series; election-night data retained 3 years).
- `eel_disenfranchisement_flags`: 7 years post-election.
- `eel_s14a_recommendations`: permanent.
- `eel_cv_estimates`: 90 days (operational monitoring only).

---

## 14. API surface

### 14.1 REST (`crates/queue-pipeline`, axum 0.8, port 8080)

- `GET /v1/queue/stations/{station_id}/snapshot`: current queue snapshot for one station.
- `GET /v1/queue/stations`: GeoJSON map view with all station queue properties.
- `GET /v1/queue/s14a-candidates`: current S.14A recommendation list (HSM-signed).
- `GET /v1/queue/anomalies`: disenfranchisement flags and queue anomaly events.
- `GET /v1/queue/wait-times`: system-wide wait time percentiles (t-digest).
- `GET /healthz` + `/readyz` + `/openapi.json`.

All endpoints unauthenticated (public). Rate limit: 1000 req/min per /64 subnet.

### 14.2 Events (CloudEvents 1.0 via outbox)

Consumed:
- `eel.biometric_edge.identification_submitted.v1`
- `eel.field_mobile.queue_reported.v1`

Emitted:
- `eel.queue.snapshot_published.v1`
- `eel.queue.disenfranchisement_flagged.v1`
- `eel.queue.s14a_recommended.v1`

### 14.3 LiveView (Elixir Phoenix, port 4000)

`ex/queue_live/`: public map; no authentication.

---

## 15. Events

Full event contracts documented in `services/queue-svc/` (data_dictionary.yaml, TRACEABILITY.yaml).

| Event | Topic | Delivery |
|---|---|---|
| eel.queue.snapshot_published.v1 | eel.queue.snapshot.v1 | at-least-once idempotent |
| eel.queue.disenfranchisement_flagged.v1 | eel.queue.alerts.v1 | at-least-once idempotent |
| eel.queue.s14a_recommended.v1 | eel.queue.s14a.v1 | exactly-once |

---

## 16. Summary

Total footprint: 1 Rust crate (queue-pipeline) + 1 Elixir LiveView app (queue_live) + 1 Python CV pipeline worker + 2 PostgreSQL migrations (V001, V002). Covers M19 (queue transparency with multi-source weighted median fusion, t-digest wait-time percentiles, disenfranchisement heuristic, S.14A HSM-signed recommendations, CV head-count via CSRNet/MCNN, and public real-time station map). Key production inputs remaining: CCTV RTSP feed credentials per station (optional; service degrades gracefully without them); HSM key provisioning for `s14a_recommender` slot; WhatsApp alert channel for voter-facing notifications (depends on M23 WhatsApp Business API approval).

### 16.1 Inter-service dependencies

| Upstream service | Data consumed | Failure mode |
|---|---|---|
| biometric-edge-svc | `eel.biometric_edge.identification_submitted.v1` (kit throughput signal) | throughput source weight set to 0; fusion continues from clerk + CV + QR sources |
| field-mobile-svc | `eel.field_mobile.queue_reported.v1` (clerk queue reports) | clerk source excluded; weight redistributed to remaining sources |
| boundary-svc | Station poll close times and geographic coordinates | S.14A window computation degraded; map coordinates unavailable; alert raised |
| ecc-svc | Receives `eel.queue.s14a_recommended.v1` | Recommendation delivered; ecc-svc dual-control state machine acts independently |
