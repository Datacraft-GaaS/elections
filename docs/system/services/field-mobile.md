# field-mobile-svc

## 1. Overview

Backend for all mobile and field-agent surfaces: the public voter-verification BFF, personnel dispatch (check-in/ping/check-out), field support tickets/warranty/diagnostics, device telemetry ingestion - plus five Flutter apps and three Python channel workers (USSD, WhatsApp, IVR).

Runtimes: four Rust crates on axum 0.8 (three with binaries, one library), five Flutter apps under `apps/`, three FastAPI Python services under `py/`. Ports bound in code: mobile-bff `0.0.0.0:8080` (`crates/mobile-bff/src/main.rs:16-17`), support-api `0.0.0.0:8082` (`crates/support-api/src/main.rs:8`), dispatch-api `0.0.0.0:8083` (`crates/dispatch-api/src/main.rs:8`). telemetry-ingest exposes a router library only - no binary or port exists in the repo.

## 2. Ownership & Boundaries

Ground truth: `docs/architecture/services/field-mobile.md`. Note: this service has no `TRACEABILITY.yaml` in its folder.

| M-ID | Slug | Primary code |
|---|---|---|
| M13 | public-verifier-app | crates/mobile-bff |
| M29 | mobile-citizen-verify | crates/mobile-bff |
| M30 | mobile-party-agent | apps/party-agent |
| M31 | mobile-poll-clerk | apps/poll-clerk |
| M32 | mobile-field-support | crates/support-api + apps/field-support |
| M33 | mobile-personnel-dispatch | crates/dispatch-api + apps/personnel |
| M34 | mobile-observer | apps/observer |

Data ownership: `verification_queries`, `participation_token_lookups`, `grievance_intake`, audit shadows, `verifier_outbox`, `ussd_sessions` (migrations V1-V6); `eel_telemetry_*` tables (V052).

Upstream deps: rts-svc, bvi-svc, transparency-svc, ecc-svc. Downstream: ecc-svc (grievances/incidents), support-admin-svc (payroll attendance). Documented events `eel.field_mobile.*`: in code only telemetry CloudEvents are built (`eel.field.telemetry_signal.v1` etc., `crates/telemetry-ingest/src/events.rs:10-13`); BFF/dispatch/support handlers log but publish nothing.

## 3. Repository Layout

```
services/field-mobile-svc/
  Cargo.toml / Cargo.lock
  crates/
    mobile-bff/        src/router.rs          # 11-route citizen verifier BFF
                       src/handlers/{verify,grievance,queue,sse_handler,clone_detect}.rs
                       src/sse.rs             # station-sharded tokio broadcast
    dispatch-api/      src/routes/{assignments,checkin,checkout,ping,status}.rs
                       tests/{dispatch_test.rs,health_test.rs}
    support-api/       src/routes/{tickets,diagnostics,warranty}.rs
                       tests/{support_test.rs,health_test.rs}
    telemetry-ingest/  src/api.rs             # 8-route ingest API + handlers
                       src/{ingest,hmac_verify,replay_defence,hash_chain,batch,frames,
                            dwell_computer,alert_engine,events,repository,metrics}.rs
                       tests/telemetry_test.rs
  apps/
    poll-clerk/ party-agent/ observer/ personnel/ field-support/   # Flutter apps;
    each has lib/src/features/*, lib/src/services/*, lib/l10n/*.arb (10-12 locales), test/
  py/
    ussd-menu/   src/ussd_menu/{fsm,service,views}.py
    wa-bot/      src/wa_bot/{main,service,views}.py, flows/{verifier,register,grievance}.py
    ivr-hook/    src/ivr_hook/{main,service}.py, tts/builder.py
  migrations/    V1__verification_queries.sql .. V6__ussd_sessions.sql, V052__mobile_telemetry.sql
  deploy/        service.yaml, kustomization.yaml, mobile-bff-deployment.yaml,
                 ussd-menu-deployment.yaml
  docs/CAPABILITIES.md
```

## 4. Capabilities & Functions

### 4.1 Mobile BFF (M13/M29/M34) - routes in crates/mobile-bff/src/router.rs:19-36

| Route | Function | Behaviour | Citation |
|---|---|---|---|
| GET `/health` | `health` | Static OK JSON. | handlers/mod.rs:16 |
| GET `/verify/v1/results` | `get_results` | Results relay placeholder ("relay from M11"). | handlers/verify.rs:12-18 |
| GET `/verify/v1/results/{scope}/{code}` | `get_results_by_scope` | Constituency/regional results relay stub. | handlers/verify.rs:20-30 |
| POST `/verify/v1/token` | `verify_token` | Accepts token_sha256; returns fixed `included:true` with stub proof ref; logs query UUIDv7. | handlers/verify.rs:32-44 |
| GET `/verify/v1/token/{sha}/proof` | `get_token_proof` | Inclusion-proof stub (`proof: null`). | handlers/verify.rs:46-55 |
| POST `/verify/v1/grievances` | `file_grievance` | Generates grievance id + ack ref; logs idempotency key; no persistence. | handlers/grievance.rs:8-24 |
| GET `/verify/v1/events` | `sse_events` | SSE stream from sharded broadcast, optional `?station=` filter, 15 s keep-alive. | handlers/sse_handler.rs:21-47 |
| GET `/verify/v1/queue/{station}` | `get_queue_snapshot` | Queue snapshot relay stub (zeros). | handlers/queue.rs:5-15 |
| GET `/verify/v1/anomalies` | `get_anomalies` | Anomaly relay stub (empty list). | handlers/verify.rs:57-62 |
| POST `/verify/v1/clone-report` | `report_clone` | Clone-app report intake (echo struct). | handlers/clone_detect.rs:10-28 |
| POST `/verify/v1/binary-integrity` | `binary_integrity_check` | Attestation event; always returns `passed: true`. | handlers/clone_detect.rs:30-48 |

SSE plumbing: `ShardedBroadcast::new/shard_for/subscribe` and `station_shard` hashing (src/sse.rs:18,25,30,44).

### 4.2 Personnel dispatch (M33) - crates/dispatch-api/src/routes/mod.rs:14-29

| Route | Function | Behaviour | Citation |
|---|---|---|---|
| GET `/healthz` | inline closure | OK JSON. | routes/mod.rs:16-19 |
| GET `/v1/dispatch/assignments/{worker_id}` | `get_assignments` | Filters assignment store by worker. | routes/assignments.rs:11-32 |
| POST `/v1/dispatch/check-in` | `check_in` | Stores WorkerCheckIn (GPS, biometric hash, signature). | routes/checkin.rs:13-52 |
| POST `/v1/dispatch/ping` | `status_ping` | Stores WorkerStatusPing (status enum + reason). | routes/ping.rs:23-58 |
| POST `/v1/dispatch/check-out` | `check_out` | Computes hours_worked from latest check-in; stores check-out. | routes/checkout.rs:25-85 |
| GET `/v1/dispatch/workers/{id}/status` | `get_status` | Latest ping or 404. | routes/status.rs:11-32 |

All state is in-process `Mutex<Vec<..>>` (src/state.rs) - no DB persistence.

### 4.3 Field support (M32) - crates/support-api/src/routes/mod.rs:12-28

| Route | Function | Behaviour | Citation |
|---|---|---|---|
| GET `/healthz` | inline closure | OK JSON. | routes/mod.rs:14-17 |
| GET/POST `/v1/support/tickets` | `list_tickets` / `create_ticket` | SLA by severity: Sev1 1h, Sev2 4h, Sev3 24h; escalation level starts at 1. | routes/tickets.rs:13-73 |
| GET `/v1/support/tickets/{id}` | `get_ticket` | Fetch by UUID or 404. | routes/tickets.rs:75-96 |
| POST `/v1/support/tickets/{id}/diagnostics` | `run_diagnostic` | Diagnostic-log submission handler. | routes/diagnostics.rs:23 |
| POST `/v1/support/warranty` | `submit_claim` | Warranty claim intake. | routes/warranty.rs:23 |

In-memory ticket store only (src/state.rs).

### 4.4 Telemetry ingestion - crates/telemetry-ingest/src/api.rs:407-417 (`router`)

| Route | Function | Behaviour | Citation |
|---|---|---|---|
| POST `/v1/telemetry/ingest` | `ingest_single` | Verifies per-device HMAC, strictly increasing monotonic counter, nonce freshness, BLAKE3 prev-hash chain; server counter-signs; persists signal. | api.rs:161-229; core logic ingest.rs:65 `ingest_signal_pure` |
| POST `/v1/telemetry/ingest-batch` | `ingest_batch_handler` | Max 1000 signals; per-item accept/reject; HTTP 207 MULTI_STATUS. | api.rs:231-311; batch.rs:23 |
| POST `/v1/telemetry/sessions` | `open_session` | Opens operator session window. | api.rs:314-335 |
| PATCH `/v1/telemetry/sessions/{id}/close` | `close_session` | Closes with constrained exit_reason (normal/timeout/revoked/duress/crash). | api.rs:338-347; domain.rs:166-185 |
| POST `/v1/telemetry/devices` | `register_device` | Registers kit/app/persona/SPIFFE ID/HSM HMAC key ref. | api.rs:349-369 |
| PATCH `/v1/telemetry/devices/{id}/revoke` | `revoke_device` | Revokes device with reason. | api.rs:370-377 |
| GET `/v1/telemetry/snapshots` | `list_snapshots` | Stub: returns empty list. | api.rs:379-385 |
| GET `/v1/telemetry/snapshots/{id}/inclusion-proof` | `get_inclusion_proof` | Returns stored inclusion proof + tlog root/index. | api.rs:387-403 |

Derived analytics: motion classification and downsampling (frames.rs:28,60), GPS dwell computation (dwell_computer.rs:22), alert evaluation engine (alert_engine.rs:49,135).

### 4.5 USSD flow (M13) - py/ussd-menu

Webhooks `POST /ussd` (Africa's Talking form-encoded) and `POST /ussd/safaricom` (service.py:29-64); both return CON/END-prefixed text. FSM `route_menu` (fsm.py:81-140): main menu "EEL Verify" -> 1 Verify Token (two-part token entry, stub inclusion check for tokens starting "VALID"), 2 Results (constituency code -> stub text), 3 Grievance (violence/fraud/other -> confirm -> done), 4 Exit. Every page is validated against the 182-char USSD budget at import time (fsm.py:27-46, FR-M13-029). The `ussd_sessions` table (V6) exists but the service is currently stateless per request.

### 4.6 WhatsApp bot - py/wa-bot

Meta webhook handshake GET `/webhook/whatsapp` with hardcoded verify token `eel-verify-token` (main.py:19-29); POST `/webhook/whatsapp` dispatches by keyword sets: verify/confirm/check -> `voter_verify_flow` (stub "REGISTERED"), grievance/complaint/problem/issue -> `grievance_intake_flow` (stub ref GRV-STUB-001), register/listed/roll -> `register_lookup_flow` (stub). Caller number is SHA-256 hashed before use as lookup key (service.py `_dispatch`).

### 4.7 IVR hook - py/ivr-hook

POST `/ivr/voice` (Africa's Talking callback, main.py:19-30) -> `IvrService.handle_call` (service.py:26): DTMF menu (1 = registration verification, 2 = polling-station lookup, 0 = repeat) answered with SSML from tts/builder.py (`build_ssml`:13, `build_menu`:29). In-memory session dict keyed by provider session id.

### 4.8 Flutter apps

Five localized apps (10-12 .arb locales each): poll-clerk (login, station dashboard, incident, EOD report, kiosk mode), party-agent (login, QR verify, countersign, dispute, check-in, duress alert), observer (login, station visit, tally verify, incident, mission export), personnel (assignment, check-in/out, status ping, incident), field-support (ticket queue, diagnostics, warranty, shift report). Offline plumbing is skeletal: `LocalDbService.init(encryptionKey)` is the only local-db method in party-agent (apps/party-agent/lib/src/services/local_db_service.dart:3-6); attestation client exists (`AttestationService.attest`, attestation_service.dart:15) as does silent duress alert (`DuressAlertService.sendSilentAlert`, features/duress_alert/duress_alert_service.dart:5). No offline queue/drain implementation found in any app.

## 5. Data Model

| Table(s) | Migration | Purpose |
|---|---|---|
| `verification_queries` | V1 | Public verification requests. |
| `participation_token_lookups` | V2 | Token verification attempts. |
| `grievance_intake` | V3 | Channel (ussd/sms/whatsapp/flutter/pwa), SHA-256 description hash (no raw PII), relay_status pending/relayed/failed, ack ref, duplicate_of. |
| `audit_verification_queries_history`, `audit_grievance_intake_history` | V4 | Audit shadows. |
| `verifier_outbox` | V5 | Transactional outbox. |
| `ussd_sessions` | V6 | Ephemeral USSD sessions: phone_hash only, step, JSONB context, 30-min TTL. |
| `eel_telemetry_devices/_sessions/_signals/_activity_frames/_geo_dwells/_alerts/_queries/_snapshots` | V052 | Device registry (unique kit_id), sessions with exit reasons, range-partitioned tamper-evident signals (monotonic_counter, prev_hash/this_hash, hmac_signature, server_signature, trace/span ids), activity frames, geo dwells, alerts, query audit, Merkle snapshots with inclusion proofs. |

Stores: telemetry-ingest uses sqlx PgPool (api.rs AppState); dispatch/support/bff use in-memory Mutex vectors only. Events produced in code: telemetry CloudEvents only (events.rs:10-13).

## 6. Operation

Build/run/test:

```bash
cd services/field-mobile-svc
cargo build && cargo test                      # all Rust crates incl. tests/
cd apps/party-agent && flutter test            # likewise for other apps
cd py/ussd-menu && pytest tests/ci/test_fsm.py
cd py/wa-bot && pytest tests                   # also ivr-hook
```

Run binaries: `cargo run -p mobile-bff` (:8080), `-p support-api` (:8082), `-p dispatch-api` (:8083). Python services run via uvicorn/FastAPI (ports not fixed in code).

Env vars: `HTTP_ADDR` (mobile-bff main.rs:16), `DB_URL` read but unused (mobile-bff main.rs:14-15). telemetry-ingest expects a PgPool injected via AppState (no env parsing).

Deploy: deploy/service.yaml (blue-green primary/mirror, rolling edges, image harbor.eel.internal/eel/field-mobile-svc); kustomization.yaml lists mobile-bff-deployment.yaml and ussd-menu-deployment.yaml (both exist) plus wa-bot-deployment.yaml and ivr-hook-deployment.yaml which are missing from deploy/.

Degradation/offline: no offline mode implemented server-side; Flutter apps have UI scaffolding but no sync engine. BFF relays degrade to static stubs.

## 7. Safeguards & Security

Auth posture: none of the HTTP surfaces authenticate requests in code - no auth middleware, token check, or SVID validation exists on BFF/dispatch/support routes. The WhatsApp webhook verify token is hardcoded (wa-bot/main.py:23). This contradicts CAPABILITIES sections 5/8 (Play Integrity/App Attest, MDM attestation), which describe controls not present server-side.

Telemetry safeguards (the strongest surface):

- Per-signal HMAC-SHA256 verification against device key; failures counted in metrics (ingest.rs:81-91; hmac_verify.rs).
- Replay defence: nonce dedup within a 5-minute window (ingest.rs module header line 4; replay_defence.rs InMemoryNonceStore; production Valkey adapter documented as pending).
- Monotonic counter must be strictly greater than last stored counter or signal is rejected (ingest.rs:93-104).
- BLAKE3 hash chain per device with server counter-signature over this_hash (hash_chain.rs:12,37; ingest.rs:139-141).

PII handling: USSD stores phone_hash only (V6 line 4); grievance intake stores description_hash only (V3 lines 5-6); WA bot hashes caller numbers before lookup (service.py `_dispatch`). Check-in payloads carry biometric_hash and GPS coordinates in memory (checkin.rs:27-38).

Idempotency/rate limiting: grievance handler logs an idempotency key but performs no dedup (handlers/grievance.rs:15); batch size capped at 1000 (api.rs:235-238); no rate limiting anywhere.

Audit logging: tracing info/warn logs in every handler; audit-shadow tables exist (V4) but nothing writes them from code.

Tests verifying safeguards: crates/telemetry-ingest/tests/telemetry_test.rs (114 test references covering HMAC, replay, counters, chain, alerts); dispatch_test.rs, health_test.rs (dispatch-api), support_test.rs, health_test.rs (support-api), verify_test.rs (mobile-bff); py tests test_fsm.py, test_wa_bot.py, test_ivr_hook.py; Flutter auth_test.dart and widget_test.dart.

### Known deferrals (verified in code)

- mobile-bff verify/token/results/anomalies/queue endpoints are stubs returning fixed data (see section 4.1 citations).
- `binary_integrity_check` always passes (clone_detect.rs:44).
- Dispatch and support APIs have no persistence; documented Kafka write-behind, Valkey HyperLogLog counting, and payroll trigger (checkout.rs:69 `payroll_trigger_id: None`) are unimplemented.
- telemetry snapshots listing is a stub (api.rs:379-385).
- kustomization references two deployment manifests that do not exist.
- No TRACEABILITY.yaml for this service despite repo convention.

## 8. Observability

Health endpoints: GET /health (BFF), GET /healthz (dispatch, support), GET /health (ussd-menu service.py:24), GET /healthz (wa-bot main.py:14, ivr-hook main.py:14). Logging: JSON tracing in mobile-bff (main.rs:6-12); tracing::fmt::init elsewhere; Python logging to stderr. Metrics: telemetry-ingest has Prometheus counters via metrics.rs (HMAC failures etc., referenced at ingest.rs:89); other crates register no metrics. Tracing fields (trace_id/span_id) are carried end-to-end in telemetry signals (V052 columns; api.rs DTOs).
