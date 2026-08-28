# biometric-edge-svc

## 1. Overview

Firmware-adjacent backend and protocol logic for the KIEMS biometric kits: kit attestation
(Play Integrity / App Attest / TPM / legacy software), kit-facing enrolment and
identification ingestion, config + register-extract distribution, offline sync, OTA update
checks, heartbeats, silent distress alerts, a certificate-transparency log for kit
certificates, and the diaspora embassy voting-station protocol. Backed by Flutter kit apps
(bvr-kit, bvi-kit, diaspora-kit, legacy-android shim). Tier T1, 99.99% availability.

- **Runtime**: Rust (axum) + Flutter (Dart apps, manifests only in repo) + Django admin.
- **Listen ports**: edge-api binds `0.0.0.0:8090` (crates/edge-api/src/main.rs — hardcoded).
  deploy/service.yaml declares containers edge-api :8090 and kit-attest :8091. Note:
  README says port 8080 — contradicts code; 8090 is authoritative.
- **Entrypoint**: crates/edge-api/src/main.rs. kit-cert-transparency has no binary of its
  own; its router is mounted by a host service (not wired in-repo).

## 2. Ownership & Boundaries

From services/biometric-edge-svc/TRACEABILITY.yaml:

| M-ID | Module | Artifacts |
|---|---|---|
| M01 | bvr-device | apps/bvr-kit; crates/edge-api (enrolment, mrz endpoints) |
| M05 | bvi-device | apps/bvi-kit; crates/edge-api (identification, config, sync, ota endpoints) |

Edge cases mapped in TRACEABILITY.yaml: EC-IDN-08 distress alert (routes/distress.rs),
EC-KIT-01 GNSS/IMU spoof guard (apps/bvi-kit gnss_guard.dart), EC-TIM-01 time-fence FSM,
EC-OPS-04 dual-control emergency close, EC-HUM-01 s.44A dual-officer sign-off,
EC-HUM-02 amputee workflow, EC-HUM-05 assisted capture, EC-LEG-03 complementary mechanism.

OpenAPI binding (packages/openapi-contracts/biometric-edge/openapi.yaml): POST
/api/v1/biometric-edge/enrolments -> routes/enrolment.rs `submit_enrolment`; POST
/api/v1/biometric-edge/identifications -> routes/identification.rs `identify`; GET
/api/v1/biometric-edge/config -> routes/config.rs `get_config_default`; POST .../attestation
-> routes/attestation.rs `attest`; POST .../heartbeat -> routes/telemetry.rs `telemetry`.

Outbox events declared: kit_heartbeat.v1 (telemetry.rs), kit_distress.v1 (distress.rs),
kit_ota_delivered.v1 (ota.rs), kit_bvr_enrolled.v1 (enrolment.rs). Data ownership: kit
attestation verdicts, heartbeats, OTA manifests, edge outbox/audit tables; diaspora
stations/rolls/pouch custody. Upstream: bvr-svc (BVR_SVC_URL), bvi-svc (BVI_SVC_URL),
ecc-svc (ECC_SVC_URL) via state.rs:15-21.

## 3. Repository Layout

```
services/biometric-edge-svc/
  Cargo.toml            # workspace: edge-api, kit-attest, diaspora, sensor-driver, integration-tests
  TRACEABILITY.yaml     # WP-3.8 modules M01/M05 + edge cases + OpenAPI binding
  data_dictionary.yaml
  migrations/
    V1__kit_attestations.sql   # kit_attestations
    V2__kit_heartbeats.sql     # kit_heartbeats (partitioned)
    V3__kit_ota_manifests.sql  # kit_ota_manifests
    V4__kit_telemetry_outbox.sql # edge_outbox + edge_audit
  crates/
    edge-api/           # Kit-facing axum API (:8090); routes/* per capability
    kit-attest/         # Attestation payload verification -> AttestationVerdict
    kit-cert-transparency/ # CT log for kit certificates (append, inclusion proofs, STH)
    diaspora/           # Embassy station protocol: rolls, Form34A-D, pouch custody, tz handling
    sensor-driver/      # Sensor abstraction (src/sensor_driver.rs)
    biometric-edge-integration-tests/ # Harness; tests/integration/kit_simulator.rs virtual kits
  apps/
    bvr-kit/            # Flutter enrolment app (pubspec.yaml + README; Dart sources not present)
    bvi-kit/            # Flutter identification app (same)
    diaspora-kit/       # Flutter diaspora station app (same)
    legacy-android/     # Legacy Android shim (PARITY.md tracks gaps)
  django/diaspora-admin/ # Embassy registry console (apps/embassies with migration 0001)
  deploy/service.yaml   # edge-api + kit-attest containers, config env
  docs/                 # CAPABILITIES, EVENTS, SLA, THREAT-MODEL
  tests/integration/kit_simulator.rs
```

## 4. Capabilities & Functions

### 4.1 edge-api routes (crates/edge-api/src/routes/mod.rs:15 merges all routers)

| Route | Handler | Behavior |
|---|---|---|
| GET /healthz | health.rs:8 `health` | JSON liveness |
| POST /api/v1/biometric-edge/enrolments and alias /api/v1/edge/enrolment/upload | enrolment.rs:38 `submit_enrolment` | Validates biometric_ref non-empty (:41) and quality_score > 0 (:43); accepts device Ed25519 sig + national_id_hash fields; returns 202 enrolment_id. Doc comment: proxies to bvr-svc and emits kit_bvr_enrolled.v1 |
| POST /api/v1/biometric-edge/identifications | identification.rs:31 `identify` | 202 accepted (stub body handling) |
| POST /api/v1/edge/mrz/parse | mrz.rs:32 `parse_mrz` | Parses MRZ (TD3-style); 400 on empty input (test mrz_parse_rejects_empty) |
| POST /api/v1/biometric-edge/attestation and alias /api/v1/edge/attestation/verify | attestation.rs:27 `attest` | Delegates to kit_attest::verify_attestation; returns tier, integrity_level, expiry, cert thumbprint, legacy_warning; 403 on tampered/expired, 400 unsupported |
| GET /api/v1/biometric-edge/config | config.rs:33 `get_config_default` | Returns ConfigBundle |
| GET /api/v1/edge/register-extract (route at config.rs:27) | get_register_extract (:46) | Cached register extract for offline kits |
| POST /api/v1/edge/sync/pull | sync.rs:41 `sync_pull` | Offline queue pull, 200 |
| POST /api/v1/edge/sync/ack | sync.rs:58 `sync_ack` | Ack batch, 200 |
| POST /api/v1/edge/ota/check | ota.rs:30 `ota_check` | Compares current_version against hardcoded latest per kit_tier (bvr-kit/bvi-kit 1.0.1, legacy-android 1.0.0); returns download_url + manifest_sig ("stub-ed25519-ota-sig") when update available |
| POST /api/v1/biometric-edge/heartbeat and alias /api/v1/edge/telemetry | telemetry.rs:37 `telemetry` | Requires device_serial; 202 with server_time; persistence + outbox emit commented as production step |
| POST /api/v1/edge/distress | distress.rs:34 `distress` | Always 202 {received:true}; logs warn "DISTRESS ALERT RECEIVED - routing to ECC"; errors suppressed so kit UI never sees failure |

### 4.2 kit-attest (crates/kit-attest)

- `AttestationKind`, `AttestationPayload` (src/verify.rs:10,19).
- `verify_attestation` (verify.rs:50): rejects empty token as Tampered; maps kind ->
  tier/integrity level (PlayIntegrity/AppAttest -> MeetsStrong, TpmQuote -> MeetsDevice,
  LegacySoftware -> MeetsBasic + legacy_warning=true); derives thumbprint; 24h expiry.
- Per-kind verifiers verify_play_integrity/verify_app_attest/verify_tpm_quote/
  verify_legacy_software (verify.rs:88+) are structured stubs rejecting tokens prefixed
  "TAMPERED:"/"INVALID:"; production API calls documented in comments.
- Verdict model KitTier/IntegrityLevel/AttestationVerdict (src/verdict.rs:7,21,32).
- Tests: tests/attest_integration.rs; route tests tampered_attestation_returns_403
  (routes/attestation.rs:81).

### 4.3 kit-cert-transparency (crates/kit-cert-transparency)

- Append-only log: `KitCertTransparencyLog::append` (src/log.rs:68), `current_sth` (:86),
  `inclusion_proof` (:106), `verify_inclusion` (:126), `consistency_proof` (:160),
  `root_at` (:185); Merkle-based SignedTreeHead (log.rs:33).
- Entries: KitCertEntry::to_leaf_bytes (src/entry.rs:40).
- HTTP API (mounted under host prefix): POST /v1/kct/append, GET
  /v1/kct/inclusion-proof/{cert_hash}, GET /v1/kct/sth (src/api.rs:126-130 router).
- Issuance hook: `intercept` (src/issuance_hook.rs:51) wraps certificate issuance so every
  kit cert is logged before delivery (IssuanceReceipt :39).
- Self-check: `verify_own_cert` (src/verifier.rs:36), `verify_inclusion_proof` (:81),
  `confirm_cert_in_log` (:94) — kit-side boot verification of its own cert's inclusion.
- Tests: tests/kct_test.rs.

### 4.4 diaspora (crates/diaspora)

Router (src/api.rs:37): stations CRUD/open/close (/stations, /stations/{id}/open|close),
rolls import/freeze/get (/rolls/import, /rolls/{id}/freeze), forms 34A create/get
(/forms/34a), transmission batches (/transmission/batches), pouch custody
(/pouches, /pouches/{id}/transfer|receive).
Key functions: DiasporaRoll freeze + snapshot_hash (roll.rs:23, :38 — canonical sort then
hash); CSV roll import per embassy profile (import.rs:91 import_roll_csv; five profiles
listed at import.rs:44); transmission batching with retry backoff
(transmission.rs:48 batch_transmit, :71 next_retry_at, :83 attempt_delivery);
PouchCustody seal/transfer/receive_at_nairobi (domain.rs:128-137);
timezone close computation station_close_utc (tz.rs:53).

### 4.5 Background/backgrounded work

No daemons in-tree: outbox relay, heartbeat persistence, ECC webhook dispatch are
documented as production steps in handler comments (telemetry.rs:47, distress.rs:42-43).
Integration harness tests/integration/kit_simulator.rs simulates fleet behaviour.

## 5. Data Model

Migrations (services/biometric-edge-svc/migrations/):

| Table | Key columns | Purpose |
|---|---|---|
| kit_attestations (V1) | device_serial, attestation_kind play_integrity/app_attest/tpm_quote/legacy_software, integrity_level MEETS_STRONG/DEVICE/BASIC, kit_tier, cert_thumbprint, legacy_warning, expires_at, revoked(+reason) | Attestation verdict cache with revocation |
| kit_heartbeats (V2) | device_serial, station_code, gps_lat/lon, uptime_s, enrolled_count; default partition | Heartbeat time series |
| kit_ota_manifests (V3) | kit_tier, version, download_url, manifest_sig, active | OTA manifest registry |
| edge_outbox (V4) | event_type, aggregate_id, payload JSONB, published_at | Transactional outbox |
| edge_audit (V4) | event_type, actor_svid, payload | Audit trail |

diaspora crate migrations/crates/diaspora/migrations/001_initial.sql defines embassy
station/roll/pouch tables (see crate README). Django diaspora-admin adds embassies table
(apps/embassies/migrations/0001_initial.py).

Kafka topics produced (README, EVENTS doc): eel-edge-enrolment
(biometric-edge.enrolment.submitted.v1), eel-edge-identification
(identification.submitted.v1), eel-edge-kit (kit.attested.v1), eel-edge-diaspora
(diaspora.roll.submitted.v1). Consumed: bvr.register.frozen.v1 (register extract caching),
bvi.kit.cert.revoked.v1 (revocation push). Code emits via edge_outbox rows; no direct
producer found in handlers (deferral).

## 6. Operation

```bash
cargo test -p edge-api
cargo test -p kit-attest
cargo test -p kit-cert-transparency
cargo test -p diaspora
cargo run -p edge-api                 # binds 0.0.0.0:8090
cd apps/bvr-kit && flutter build apk --release
cd django/diaspora-admin && python manage.py test
```

Env vars (grep verified): BVR_SVC_URL, BVI_SVC_URL, ECC_SVC_URL (state.rs:15-21, defaults
defined inline). README also lists EEL_DATABASE_URL, EEL_KAFKA_BROKERS,
EEL_EDGE_OFFLINE_QUEUE_MAX_DAYS, OTEL_EXPORTER_OTLP_ENDPOINT — not referenced by any
current Rust source (not verified / aspirational).

Deploy: deploy/service.yaml — two containers (edge-api :8090, kit-attest :8091), image
harbor.eel.internal/eel/biometric-edge-svc, env from biometric-edge-svc-config ConfigMap.
README references loadtest/ virtual kit simulator (65k devices); only the single
tests/integration/kit_simulator.rs exists in-tree.

Degradation modes: kits queue up to 7 days offline and sync on reconnect (README SLA;
sync endpoints above). Distress path never fails the caller. No circuit breakers found in
code.

## 7. Safeguards & Security

- **Attestation gate**: all attestation traffic flows through
  kit_attest::verify_attestation (kit-attest/src/verify.rs:50); tampered/expired tokens
  yield 403 via ApiError mapping in routes/attestation.rs:29-35. Legacy-software kits are
  flagged legacy_warning=true and tier=MeetsBasic rather than silently trusted.
- **Input validation**: enrolment requires biometric_ref and quality_score > 0
  (enrolment.rs:40-45); telemetry/distress/ota require device_serial / known kit_tier
  (telemetry.rs:39, ota.rs:32-38); MRZ empty-input 400 (mrz.rs tests).
- **Error surfaces**: unified ApiError enum -> problem-style JSON with status mapping
  (error.rs IntoResponse); internal messages exposed as "Internal" 500.
- **Distress opsec**: response is constant {received:true} regardless of downstream state;
  failures suppressed to prevent UI leak (distress.rs:37-44 comment).
- **CT log**: append-only Merkle log with signed tree heads, inclusion + consistency
  proofs (log.rs); issuance hook guarantees certificates are logged before use
  (issuance_hook.rs:51); kit self-verifies boot cert inclusion (verifier.rs:36).
- **Audit**: edge_audit table records actor_svid per event (migrations/V4) — SPIRE SVID
  identity anticipated but no SVID check middleware exists in edge-api today (no auth on
  any route; largest gap).
- **Idempotency**: EdgeEnrolmentRequest carries idempotency_key (enrolment.rs:19) but no
  dedupe store consults it yet (deferral).
- **OTA integrity**: manifests carry manifest_sig; current value is stub-ed25519-ota-sig
  (ota.rs:52) — signature verification pending (deferral).
- **Constant-time crypto**: none implemented in this service (thumbprint derivation is a
  plain hash stub); Ed25519 device signatures are transported but not verified server-side
  yet (device_sig field accepted unverified — deferral).
- **Biometric template protection**: templates are referenced by content address
  (biometric_ref = v1:sha256:...) and proxied to bvr-svc; no raw biometrics persisted by
  this service (schema contains none). HSM/enclave boundaries live in abis-svc/rts-svc.
- **Tests verifying safeguards**: routes/*.rs #[cfg(test)] suites (e.g.
  tampered_attestation_returns_403, distress_returns_202 matching EC-IDN-08,
  mrz_parse_rejects_empty, ota_check_update_available, sync/heartbeat happy paths);
  kit-attest/tests/attest_integration.rs; kit-cert-transparency/tests/kct_test.rs;
  diaspora/tests/diaspora_test.rs; tests/integration/kit_simulator.rs.
- **Known deferrals**: verify_play_integrity/app_attest/tpm_quote call-outs to Google/
  Apple/TPM CA are stubs (verify.rs comments); heartbeat persistence +
  kit_heartbeat.v1 emission commented out (telemetry.rs:47); kit_distress.v1 outbox +
  ECC webhook pending (distress.rs:42); ota manifest signing stub; register-extract
  endpoint serves static bundle; Flutter app sources absent (pubspec/README only);
  README-documented `/v1/edge/kits/{serial}/...` endpoint family does not match the
  implemented `/api/v1/...` routes.

## 8. Observability

- Liveness: GET /healthz returns JSON status (routes/health.rs:8) with unit test.
- Tracing: tracing_subscriber init in main.rs; structured spans per handler
  (tracing::warn! for distress, tracing::debug! for heartbeats with battery_pct).
- Metrics/tracing export: OTEL_EXPORTER_OTLP_ENDPOINT named in README only; no metrics
  exporter in code (not verified).
- Health surface for orchestration: deploy/service.yaml exposes :8090; readiness probes
  not defined in the manifest beyond the port declaration.
