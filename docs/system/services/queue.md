# queue-svc

## 1. Overview

Queue Transparency Service (module M19): fuses multi-source polling-station queue signals into per-station snapshots, estimates wait times, flags disenfranchisement risk, and produces Section 14A (election-hours extension) recommendations for the ECC. Also serves the public queue map (Phoenix LiveView + SSE) and a computer-vision headcount worker.

Runtimes: Rust axum binary `eel-queue-pipeline` (crates/queue-pipeline, default port 4110), Elixir/Phoenix LiveView app `ex/queue_map` (dev port 4111, test 4112), Python FastAPI `py/cv-headcount`. Entrypoints: crates/queue-pipeline/src/main.rs:4 (`main`), ex/queue_map/lib/queue_map_live/application.ex:6, py/cv-headcount/src/cv_headcount/service.py:14.

## 2. Ownership & Boundaries

Ground truth: docs/architecture/services/queue.md and services/queue-svc/TRACEABILITY.yaml (module M19, work package WP-4.2).

| M-ID | Requirement | Implementation | Test |
|---|---|---|---|
| M19-R01 | Snapshot fusion from kit/clerk/CV/QR sources | pipeline.rs::fuse_snapshot | pipeline.rs tests::test_fuse_snapshot |
| M19-R02 | Wait-time breach / disenfranchisement flagging | pipeline.rs::check_wait_breach | tests::test_wait_breach |
| M19-R03 | s.14A recommendation with Chair HSM sign gate | pipeline.rs::recommend_s14a | tests::test_s14a_recommended |
| M19-R04 | Public queue map via LiveView SSE | ex/.../station_map_live.ex | station_map_live_test.exs |
| M19-R05 | CV headcount from camera feeds | py/cv-headcount service.py | tests/ci/test_cv_headcount.py |

Data owned: `eel_queue_snapshots`, `eel_queue_events` (migrations). Upstream deps: biometric-edge kit signals, field-mobile queue reports, CCTV feeds; downstream: public web map, ECC. Documented events: `eel.queue.snapshot.v1`, `eel.queue.wait_time_breached.v1`, `eel.queue.disenfranchisement_flag.v1`, `eel.queue.s14a_recommended.v1` - all four are constructible in code but none are persisted or published by any handler.

## 3. Repository Layout

```
services/queue-svc/
  Cargo.toml / Cargo.lock
  crates/queue-pipeline/
    src/main.rs        # binary entrypoint, HTTP_ADDR env, default :4110
    src/router.rs      # 2 routes
    src/handlers.rs    # health + snapshot fusion handler
    src/pipeline.rs    # fuse_snapshot, breach/flag/s14a checks + unit tests
    src/models.rs      # QueueSnapshot and event structs
  ex/queue_map/
    lib/queue_map_live/router.ex             # LiveView "/", SSE, health scopes
    lib/queue_map_live/live/station_map_live.ex
    lib/queue_map_live/sse_controller.ex
    lib/queue_map_live/controllers/health_controller.ex
    config/{config,dev,test}.exs             # ports 4111 dev / 4112 test
    test/queue_map_live/station_map_live_test.exs
  py/cv-headcount/
    src/cv_headcount/{service,views,detector}.py
    tests/ci/test_cv_headcount.py
  migrations/V001__queue_snapshots.sql, V002__queue_events.sql
  loadtest/k6_queue_load.js
  tests/ci/test_pipeline_integration.rs
  deploy/service.yaml
  data_dictionary.yaml, TRACEABILITY.yaml, docs/CAPABILITIES.md
```

## 4. Capabilities & Functions

### 4.1 Queue pipeline REST API - crates/queue-pipeline/src/router.rs:5-10

| Route | Function | What it does | Citation |
|---|---|---|---|
| GET `/v1/queue/health` | `health` | Returns {"ok": true}. | handlers.rs:12-14 |
| POST `/v1/queue/stations/{station_code}/snapshot` | `post_snapshot` | Fuses submitted signal counts into a QueueSnapshot and returns it synchronously; no DB write, no event emission. | handlers.rs:16-23 |

Fusion logic (pipeline.rs):

- `fuse_snapshot` (line 12): max-headcount across CV/clerk/QR sources; wait = headcount * 60 / kit throughput; disenfranchisement flag when wait > 45 min (`WAIT_DISENFRANCHISEMENT_THRESHOLD`, line 9) or any source > 200 (`HIGH_THROUGHPUT_THRESHOLD`, line 10); s14a recommendation mirrors the flag (line 34).
- `check_wait_breach` (line 50): returns `WaitTimeBreachedEvent` when estimate exceeds caller threshold.
- `check_disenfranchisement` (line 68): returns `DisenfranchisementFlagEvent` when flagged.
- `recommend_s14a` (line 85): returns `S14aRecommendedEvent` with `hsm_pending: true`; no HSM call is made in code.
- Event wrapper `QueueSnapshotEvent::new` types events as `eel.queue.snapshot.v1` (models.rs:28-36); never constructed by handlers.

Unit tests inline at pipeline.rs:100-197; integration tests tests/ci/test_pipeline_integration.rs:6-40.

### 4.2 Public queue map (LiveView) - ex/queue_map

| Surface | Function | Behaviour | Citation |
|---|---|---|---|
| LiveView `/` | `StationMapLive.mount/handle_info/render` | Subscribes to PubSub topic `queue:updates`; merges `{:queue_update, payload}` messages into station list; renders urgency cards green (<15 min), amber (<30), red (>=30) via `urgency_class` (line 64) and labels (line 68). Seed data is hardcoded `sample_stations` (lines 72-78). | live/station_map_live.ex:12-78 |
| SSE GET `/sse/queue-updates` | `SseController.queue_updates` | Chunked text/event-stream; subscribes to PubSub, encodes payloads as `event: queue_update`; 30 s keepalive comment. | sse_controller.ex:7-40 |
| GET `/health/` | `HealthController.index` | {"ok": true, service: "queue_map_live"}. | controllers/health_controller.ex:4-6 |

Router pipelines with CSRF protection and secure browser headers for the LiveView scope (router.ex:4-35).

### 4.3 CV headcount worker - py/cv-headcount

| Route | Function | Behaviour | Citation |
|---|---|---|---|
| GET `/v1/cv/health` | `health` | {"ok": true}. | service.py:34-36 |
| POST `/v1/cv/headcount` | `headcount` | Calls detector; returns result_id (UUID7), headcount, confidence, source, timestamp. Pydantic models forbid extra fields. | service.py:39-52; views.py:8-21 |

Detector (detector.py:25-41): deterministic stub. Only `fixture://` image sources produce output (seeded RNG from station_code hash); every other source returns `(0, 0.0)` pending real CV integration - the documented ONNX model does not exist.

## 5. Data Model

From migrations:

| Table | Columns | Purpose |
|---|---|---|
| `eel_queue_snapshots` (V001 lines 1-14) | id UUID PK, station_code, kit_throughput_per_hour, clerk_report_count, cv_headcount, qr_scan_count, estimated_wait_minutes, disenfranchisement_flag, s14a_recommended, fused_at, signature, created_at | Per-station fused snapshot; indexes on station_code and fused_at DESC. |
| `eel_queue_events` (V002 lines 1-7) | id UUID PK, event_type, station_code, payload JSONB, occurred_at | Event archive; indexes on (station_code, occurred_at DESC) and (event_type, occurred_at DESC). |

Contradiction note: no code path writes either table - the pipeline handler is pure compute (handlers.rs:16-23) and the LiveView reads hardcoded samples. The schema matches the documented read-model design but is currently unused by this repo's code.

Events produced (constructed only, not published): the four `eel.queue.*` types above (models.rs:28-36, pipeline.rs:55-97). Events consumed: none implemented (documented Kafka consumers absent).

## 6. Operation

Build/run:

```bash
cd services/queue-svc
cargo run -p eel-queue-pipeline            # HTTP on $HTTP_ADDR or 0.0.0.0:4110
cd ex/queue_map && mix deps.get && mix phx.server   # dev server on 127.0.0.1:4111
cd py/cv-headcount && uvicorn cv_headcount.service:app
```

Test:

```bash
cargo test                                  # pipeline units + integration test
cd ex/queue_map && mix test
cd py/cv-headcount && pytest tests/ci
k6 run loadtest/k6_queue_load.js
```

Env vars: `HTTP_ADDR` (pipeline main.rs:14, default 0.0.0.0:4110); RUST_LOG-style filter directive `eel_queue_pipeline=info` (main.rs:9). Ports: 4110 (pipeline), 4111 (queue_map dev), 4112 (queue_map test).

Deploy: deploy/service.yaml - image harbor.eel.internal/eel/queue-svc, blue-green primary/mirror (3 replicas), rolling at five edge sites (2 replicas). No manifests for the Elixir map or CV worker are present.

Degradation: none implemented. Documented fallbacks (CV feed loss -> identification-rate proxy; stale snapshots do not block S.14A; CAPABILITIES section 2 flows) are not present in code.

## 7. Safeguards & Security

Auth posture: all three surfaces are unauthenticated by design intent for the public map, but the snapshot POST endpoint also has no authentication or SVID check (router.rs:6-9; handlers.rs:16) - any client can submit fusion inputs.

Input validation: `SignatureFusionInput` is deserialized as u32 counts (models.rs:67-73), so negative values are rejected by serde; there is no upper bound or plausibility validation. The CV endpoint validates strictly via Pydantic `extra="forbid"` and confidence constrained 0.0-1.0 (views.py:9,20).

Idempotency/rate limiting: none implemented on any surface.

PII handling: queue data contains no personal identifiers - station codes and aggregate counts only; CV worker receives image source strings, not images.

Audit logging: structured tracing logs (post_snapshot handlers.rs:20; Elixir Logger for queue updates and SSE connects station_map_live.ex:24, sse_controller.ex:16). The `signature` column exists on snapshots (V001 line 12) but nothing signs snapshots; `S14aRecommendedEvent.hsm_pending=true` documents the pending Chair HSM sign gate rather than enforcing it (pipeline.rs:91).

Tests verifying safeguards: pipeline threshold/breach tests (pipeline.rs:100-197), cross-crate integration assertions (tests/ci/test_pipeline_integration.rs), LiveView test (ex/queue_map/test/queue_map_live/station_map_live_test.exs), CV CI tests (tests/ci/test_cv_headcount.py).

### Known deferrals (verified against code)

- No persistence layer wired into the pipeline despite migrations existing.
- No Kafka consumer/producer despite TRACEABILITY.yaml listing four produced event types and README describing consumption of kit and field-mobile topics.
- HSM signing gate for S.14A recommendations is represented only by an `hsm_pending` boolean.
- CV headcount is a fixture-only stub (detector.py:25-33); ONNX model absent.
- StationMapLive serves three hardcoded stations (station_map_live.ex:72-78); PubSub producers are not implemented anywhere in-repo.
- Documented REST endpoints GET /v1/queue/stations, /s14a-candidates, /anomalies (README Key APIs) do not exist.

## 8. Observability

Health endpoints: GET /v1/queue/health (handlers.rs:12), GET /health/ (health_controller.ex:4), GET /v1/cv/health (service.py:34). Logging: JSON tracing subscriber in the pipeline (main.rs:6-12); console Logger with request_id metadata in the Elixir app (config/config.exs:12-14). Metrics: none registered in any component; documented Prometheus metric names (CAPABILITIES sections) are unimplemented. Load testing harness exists at loadtest/k6_queue_load.js.
