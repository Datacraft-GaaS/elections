# rts-svc

## 1. Overview

The Results Transmission System (RTS) receives, validates, tallies, and publishes election
result forms (Form 34A station / 34B constituency / 34C national) transmitted from polling
stations via kits and Returning Officer (RO) desktops. It runs ICR (handwriting recognition)
over scanned forms, aggregates 34A into 34B and 34C, anchors publication Merkle roots in the
transparency log, and drives the declaration/certificate workflow. Tier T1, election-critical.

- **Runtime**: Rust (axum/tokio workspaces), Python (rts-icr ML service), Django (admin UI),
  Tauri (RO desktop app). No Elixir apps exist under services/rts-svc (verified: no *.ex/*.exs).
- **Listen ports**: rts-api `0.0.0.0:8080` HTTP + gRPC 50051 (deploy only,
  services/rts-svc/deploy/base/rts-api-deployment.yaml); rts-device-api default
  `0.0.0.0:8082` (crates/rts-device-api/src/main.rs:14); rts-ro-api default
  `0.0.0.0:8083` (crates/rts-ro-api/src/main.rs:14). Projections/publisher/tally are
  background workers with no listener.
- **Entrypoints**: crates/rts-api/src/main.rs, crates/rts-device-api/src/main.rs,
  crates/rts-ro-api/src/main.rs, crates/rts-tally/src/main.rs,
  crates/rts-publisher/src/main.rs, apps/ro-desktop/src-tauri/src/main.rs.

Note: services/rts-svc/README.md describes endpoints (`/v1/rts/...`) that do not exist in
code; this document follows the actual routers.

## 2. Ownership & Boundaries

From services/rts-svc/TRACEABILITY.yaml and docs/architecture/services/rts.md:

| M-ID | Module | Scope |
|---|---|---|
| M08 | rts-device | Kit-side form scan/sign/transmit; participation tokens |
| M09 | rts-ro | RO desktop Form 34A review/amendment, disputes |
| M10 | rts-backend | Form storage, tally, Merkle anchoring, publication |
| M51 | result-declaration | Declaration workflow, Certificate of Election |
| M63 | ballot-serial-chain | Serial assignment, receipts, spoilt ballots, reconciliation |

- **Data ownership**: append-only event store `eel_events`; read models form34a/b/c_view;
  publication snapshots; declarations/certificates/petition windows; ballot serial tables.
- **Upstream**: bvr-svc (`register.frozen` gates acceptance per README), inventory-svc
  (kit config, serial ranges). Consumed topics not wired in code (stub main loops).
- **Downstream**: transparency-svc (Merkle leaf anchoring via rts-tally/src/tlog_client.rs),
  portal-svc (public snapshots), governance-svc/ecc-svc (audit/alert events).

Legal rules implemented in code: EC-DAT-10/EC-DAT-11 validators
(crates/rts-domain/src/validators.rs:13, :30) and EC-LEG-01 withdrawn/deceased candidate
handling (crates/rts-domain/src/candidate_status.rs).

## 3. Repository Layout

```
services/rts-svc/
  Cargo.toml            # local workspace (10 member crates below); zk/fhe/bft live in root workspace only
  TRACEABILITY.yaml     # WP-3.1 module M10, events, legal requirements
  data_dictionary.yaml  # Form34A/B/C entity definitions
  migrations/           # V0001 events schema, V0002 projections, V0003 audit shadows
  crates/
    rts-domain/         # Aggregates Form34A/B/C, validators, candidate status, events
    rts-api/            # Public REST API (:8080) + gRPC stubs (grpc.rs)
    rts-device-api/     # M08 kit-facing API (:8082): multipart upload, SMS fallback
    rts-ro-api/         # M09 RO workstation API (:8083)
    rts-tally/          # Kafka consumer stub, aggregators, Merkle, drift detector, tlog client
    rts-projections/    # Read-model projector runner + ro/cro/iab/portal views
    rts-publisher/      # Publication scheduler: bundle -> hash -> MinIO URL -> tlog anchor
    declaration/        # M51 declaration aggregate + REST router + gazette + proptest rules
    ballot-serial/      # M63 serial chain aggregate + REST router + reconciliation
    behavioural-biometrics/ # Keystroke/mouse session profiling, hijack detection (root workspace only)
    zk-tally/           # Groth16 tally circuit, prover, verifier, setup (root workspace only)
    fhe-tally/          # CKKS-style toy encrypted aggregation, threshold decrypt (root workspace only)
    bft-tally/          # HotStuff-style BFT replica, pacemaker, QC certificates (root workspace only)
    rts-integration-tests/ # Integration harness (excluded from root workspace, Cargo.toml:20)
  apps/
    ro-desktop/         # Tauri RO workstation (src-tauri/src/commands.rs)
    rts-kit/            # Flutter kit app shell (pubspec.yaml only, no lib/ verified)
  django/declaration-admin/ # Admin UI for declarations, certificates, holds (views.py)
  py/rts-icr/           # ICR OCR worker + drift monitor (ocr.py is a stub)
  deploy/base/          # Kustomize: rts-api, rts-tally, rts-publisher, rts-projections
  docs/                 # CAPABILITIES/EVENTS/SLA/THREAT-MODEL
  src/                  # Empty (.gitkeep); README's "cargo run --bin rts-svc" does not apply
```

## 4. Capabilities & Functions

### 4.1 rts-api REST (crates/rts-api/src/router.rs:4 `build_router`, all handlers in handlers.rs)

| Route | Handler | Behavior |
|---|---|---|
| GET /health | `health` (handlers.rs:56) | 200 OK |
| POST /api/v1/rts/forms | `submit_form` (:6) | Accepts Form34ARequest, returns 202 with generated form_id + icr_job_id (in-memory stub) |
| POST /api/v1/rts/forms/sms-fallback | `sms_fallback` (:10) | Accepts SmsFallbackRequest, 202 |
| POST /api/v1/rts/forms/34b | `compile_form34b` (:34) | 202 "compilation_queued" |
| POST /api/v1/rts/forms/34c | `compile_form34c` (:37) | 202 "compilation_queued" |
| GET /api/v1/rts/forms/{form_id}/proof | `get_form_proof` (:31) | Returns empty merkle proof + zero root (stub) |
| POST /api/v1/rts/tokens | `provision_token` (:14) | Returns stub participation token string |
| POST /api/v1/rts/events | `ingest_event` (:18) | 202 queued event_id |
| GET /api/v1/rts/acks/{scan_id} | `get_ack` (:22) | status "pending" |
| GET /api/v1/rts/snapshots | `list_snapshots` (:25) | Empty list |
| GET /api/v1/rts/snapshots/{scope}/{code} | `get_snapshot` (:28) | Empty snapshot shell |
| GET /api/v1/rts/icr/{form_id} | `get_icr_job` (:40) | Pending ICR job |
| GET /api/v1/rts/icr/{job_id}/result | `get_icr_result` (:43) | Pending result |
| GET /api/v1/rts/audit/{form_id} | `get_audit` (:46) | Empty event list |
| POST /api/v1/rts/ro/disputes | `create_dispute` (:49) | 201 dispute created |
| POST /api/v1/rts/ro/disputes/{id}/resolve | `resolve_dispute` (:53) | Resolved |

gRPC message shapes and stub functions `submit_form_event`/`query_tally`
(crates/rts-api/src/grpc.rs:19, :23) exist but no tonic server is bound; port 50051 is
declared only in deploy/base/rts-api-deployment.yaml. proto/ directory does not exist.

### 4.2 rts-device-api (crates/rts-device-api/src/router.rs; handlers.rs)

| Route | Handler | Behavior |
|---|---|---|
| POST /api/v1/rts/device/form34a/upload | `upload_form34a` (handlers.rs:15) | Multipart image (<=2 MB enforced at :16,:28) + ocr_json + signature + idempotency_key; 202 |
| POST /api/v1/rts/device/form34a/sms-fallback | `sms_fallback` (:81) | base64(?) payload decoded then CBOR+zstd via `codec::decode_sms` (src/codec.rs); 422 on decode error. Note: helper `base64_decode` (:195) is actually hex decoding — known gap |
| POST /api/v1/rts/device/transmission-attempt | `transmission_attempt` (:128) | Records attempt (log-only; outbox emit commented as stub) |
| GET /api/v1/rts/device/participation-tokens/{station} | `participation_tokens` (:151) | Stub bounded token SHA-256 shape |
| POST /api/v1/rts/device/dual-sim-resubmit | `dual_sim_resubmit` (:168) | Idempotency-Key dedupe contract documented; dedup always returns false (stub) |
| GET /health | `health` (:189) | 200 |

### 4.3 rts-ro-api (crates/rts-ro-api/src/router.rs:7; handlers.rs)

| Route | Handler | Behavior |
|---|---|---|
| GET /health | `health` (handlers.rs:179) | 200 |
| GET /api/v1/rts/ro/queue | `get_queue` (:13) | Empty queue |
| POST /api/v1/rts/ro/form34a/{id}/verify | `verify_form34a` (:23) | 202, emits-shape form34a_verified.v1 |
| POST /api/v1/rts/ro/form34a/{id}/amend | `amend_form34a` (:45) | 202, amendment lineage fields logged |
| POST /api/v1/rts/ro/form34b/compile | `compile_form34b` (:69) | 202; dual-officer RO+witness requested in body |
| POST /api/v1/rts/ro/form34c/compile | `compile_form34c` (:91) | 202; SpiceDB CRO gate is a comment-stub (:101) |
| POST /api/v1/rts/ro/disputes | `create_dispute` (:114) | 201 dispute_raised.v1 |
| POST /api/v1/rts/ro/agents/countersign | `agent_countersign` (:136) | 202 agent_countersign.v1 |
| POST /api/v1/rts/ro/session-recording | `session_recording` (:159) | start/anchor/stop; 60-s segment hashes to transparency-svc |

### 4.4 Declaration API (crates/declaration/src/api.rs:197 `router`)

POST /api/v1/declarations (`initiate_declaration`), GET/:id, PUT declare/runoff/tie,
POST amend, GET certificate, GET petition-window, PUT judiciary-ack.
Aggregate state machine in src/aggregate.rs; gazette rendering in src/gazette.rs;
property tests in src/proptest_rules.rs.

### 4.5 Ballot-serial API (crates/ballot-serial/src/api.rs:199 `router`)

POST/GET assignments, POST receipts, POST spoilt, POST reconcile,
GET reconcile/:station_id, PUT reconcile/:id/resolve (`resolve_hold`).
Reconciliation logic in src/reconciliation.rs.

### 4.6 Tally pipeline (background)

- Consumer: `TallyConsumer::run` (crates/rts-tally/src/consumer.rs:10) over topic
  `eel.rts.form34a.v1` using eel-eventstore-rs exactly-once consumer; currently logs and acks.
- Aggregation: `aggregate_34b` (aggregator.rs:7), `aggregate_34c` (aggregator.rs:36).
- Merkle: `compute_merkle_root` (merkle.rs:9), SHA-256 leaf hashing (merkle.rs:5).
- Drift: `DriftDetector::record_tally` (drift.rs:22) emits `eel.rts.parallel_tally_drift.v1`.
- Anchoring: `TlogClient::append_leaf` (tlog_client.rs:166), envelope builder
  `build_anchor_envelope` (:70), config from env `from_env` (:191).

### 4.7 Publication loop (crates/rts-publisher)

`scheduler::run_publication_loop` (scheduler.rs:16): every `PUBLICATION_INTERVAL_SECS`
(default 300) create_bundle (bundle.rs:33) from latest compiled `form34c_view` row ->
BLAKE3 content hash via `sign_bundle` (signer.rs:27) -> MinIO object URL via
`upload_to_minio` (signer.rs:44; fails loudly if `EEL_MINIO_ENDPOINT` unset) ->
`tlog.append_leaf`. Real cosign signing gated on sigstore API (comment, signer.rs:21-24).

### 4.8 Projections (crates/rts-projections)

`ProjectionRunner::run_from_store` (projector.rs:20) replays events to views:
cro_view.rs, iab_view.rs, portal_snapshot.rs, ro_view.rs.

### 4.9 ZK / FHE / BFT tally (root workspace members, HARD-B flag)

- zk-tally: `TallyCircuit` (zk-tally/src/circuit.rs:46, arkworks Bls12_381 Groth16),
  `TallyProver::prove` (prover.rs:32), `TallyVerifier::verify` (verifier.rs:27),
  `generate_params` (setup.rs:41), binary serialisation (serialise.rs:21-44).
- fhe-tally: `encrypt_34a_leaf` (encrypt_leaf.rs:52), `aggregate_encrypted_leaves`
  (aggregate.rs:42), `decrypt_tally` (decrypt.rs:83) — toy-scale implementations;
  policy gate `FhePolicy::check` (policy.rs:55); aggregation proof `AggregationProof::verify`
  (prove.rs:44).
- bft-tally: HotStuff-style `Replica::propose/handle_prepare/collect_vote/advance_phase/
  commit_block` (replica.rs:123-245), leader rotation (`leader_for_view`, replica.rs:59),
  exponential pacemaker (pacemaker.rs:29-74), Ed25519 votes (`sign_vote`, replica.rs:106);
  read API `/v1/bft/health|blocks/{height}|qc/{block_hash}` (api.rs:93-96).
- behavioural-biometrics: session features endpoint `/v1/behavioural/session/{session_id}/features`
  (behavioural-biometrics/src/api.rs:34); keystroke (keystroke.rs) and mouse (mouse.rs)
  profilers; `HijackDetector::check` (hijack_detector.rs:28).

### 4.10 Python ICR worker (py/rts-icr)

`process_job` (src/rts_icr/worker.py:12): fetches image, calls `extract_fields`
(src/rts_icr/ocr.py:7 — synthetic stub, TODO PaddleOCR ONNX), flags
`requires_review` when any field confidence < 0.90 (worker.py:9 CONFIDENCE_GATE).
`DriftMonitor.check_drift` (src/rts_icr/drift.py:24) warns when mean confidence
drifts > 0.15 from baseline 0.92 after >= 10 samples.

### 4.11 Django admin (django/declaration-admin)

URLs in declaration_admin/urls.py; views.py role-gated pages: declarations_queue (:47),
declaration_detail (:73), certificate_panel (:89), issue_certificate (:101, stub POST),
petition_window_monitor (:119), correction_workflow (:131), reconciliation_holds_queue,
discrepancy_adjudication (:211, PO+CRO dual sign-off enforced at :222-228).

### 4.12 RO desktop (apps/ro-desktop, Tauri)

Commands (src-tauri/src/commands.rs): get_queue (:24), review_form34a (:36),
compile_34b (:63), create_dispute (:104), session_recording (:123),
get_pending_actions (:142); local SQLite persistence in src-tauri/src/db.rs;
power-cut test src-tauri/tests/power_cut.rs.

## 5. Data Model

PostgreSQL migrations (services/rts-svc/migrations/):

| Table | Purpose |
|---|---|
| eel_events (V0001) | Append-only event store; UNIQUE(stream_id, stream_version), prev_hash chain, producer_signature + hsm_countersignature columns |
| eel_snapshots (V0001) | Aggregate snapshots keyed (stream_id, stream_version) |
| eel_outbox (V0001) | Transactional outbox; partial index on unpublished |
| form34a_view / form34b_view / form34c_view (V0002) | Read models: counts, candidate_totals JSONB, status, merkle_root (34c) |
| publication_snapshots (V0002) | snapshot_id, form34c FK, merkle_root, signed_url, tlog_leaf_index |
| audit.form34a_view_history (V0003) | Trigger-populated audit shadow (audit.log_form34a_changes) |

ballot-serial (crates/ballot-serial/migrations/V0001): serial_assignments (unique per
station), ballot_issuances (receipt_hash), ballot_spoils, reconciliations
(status pending/held/cleared/flagged), discrepancy_holds (PO+CRO resolution columns),
ballot_serial_audit_shadow, ballot_serial_outbox.

declaration (crates/declaration/migrations/V0001-V0002): declarations
(status incl. runoff_required/tie_pending/annulled, hsm_signature), certificates_of_election
(pdf_minio_ref, tlog_leaf_index), petition_windows (T+28d close, EC-TIM-04),
judiciary_acks, declaration_outbox, declaration_amendments (7-year retention,
previous/new_snapshot_hash), declaration_audit_shadow.

Kafka topics produced (TRACEABILITY.yaml): `eel.rts.form34a.v1`, `eel.rts.form34b.v1`,
`eel.rts.form34c.v1`, `eel.rts.publication.v1`, `eel.rts.audit.v1`.
Consumed: `eel.rts.form34a.v1` by rts-tally (consumer.rs:5 TOPIC_FORM34A).

## 6. Operation

```bash
# Build/test a crate (from repo root workspace or service dir)
cargo test -p rts-domain
cargo test -p rts-tally
cargo test -p declaration
cargo test -p ballot-serial
cargo nextest run -p rts-integration-tests   # harness exists; excluded from root ws (Cargo.toml:20)

# Run binaries
cargo run -p rts-api            # binds 0.0.0.0:8080
HTTP_ADDR=0.0.0.0:8082 cargo run -p rts-device-api
HTTP_ADDR=0.0.0.0:8083 cargo run -p rts-ro-api
DB_URL=postgres://... cargo run -p rts-publisher
```

Env vars (grep of env::var): DB_URL (rts-api main.rs:10, rts-tally main.rs:12 required,
rts-publisher main.rs:8 required), KAFKA_BROKERS + TLOG_ENDPOINT (rts-tally main.rs:10-11),
HTTP_ADDR (device/ro api), PUBLICATION_INTERVAL_SECS (publisher main.rs:9),
EEL_MINIO_ENDPOINT / EEL_MINIO_BUCKET (signer.rs:35-36),
EEL_TLOG_SIGNING_KEY / EEL_TLOG_KEY_ID / EEL_ELECTION_ID / EEL_HLC_NODE
(tlog_client.rs:191 from_env), RTS_RO_API_URL (ro-desktop lib.rs).

Deploy artifacts: deploy/base/kustomization.yaml with rts-api (3 replicas, ports 8080+50051),
rts-tally, rts-publisher, rts-projections Deployments; DB_URL sourced from rts-db-secret.
README claims overlays (primary/bomas/edge sites) not present in deploy/ (not verified).

Degradation modes (docs/architecture/services/rts.md): writes fail-fast without PG primary;
ICR degrades to manual RO entry (`pending-icr`); publication halts when HSM unreachable;
anchoring retries with backoff + stale-anchor alert. Publisher degrades gracefully today:
tlog=None disables anchoring with an explicit log line each cycle (scheduler.rs:25).

## 7. Safeguards & Security

- **Arithmetic validation**: validate_ec_dat_10 (validators.rs:13) enforces zero-results
  rejection + votes_cast = valid + rejected + spoilt; validate_ec_dat_11 (validators.rs:30)
  enforces rejected+valid <= registered. Tests: crates/rts-domain/tests/validators_test.rs.
- **Event integrity**: eel_events.prev_hash chaining, producer_signature,
  hsm_countersignature columns (migrations/V0001__events_schema.sql).
- **Idempotency**: device upload accepts idempotency_key field (device handlers.rs:47-50);
  dual-sim resubmit contract documented (:165-187); ro handlers log idempotency keys.
  No server-side dedupe store is implemented yet (deferral).
- **Input size limits**: 2 MB image cap enforced with 413 (device handlers.rs:16,28-34).
  Total-request 4 MB cap documented at :14 but not separately enforced (deferral).
- **Role gating**: Django role_required decorator (declaration-admin/views.py:26) with
  Chair/CRO/RO groups; dual-officer enforcement for discrepancy adjudication
  (views.py:222-228). Rust-side CRO SpiceDB gate is a stub (rts-ro-api handlers.rs:101).
- **Audit logging**: audit shadow triggers (V0003), ballot_serial_audit_shadow +
  declaration_audit_shadow tables; tracing::info JSON logs on every mutating handler.
- **Crypto**: BLAKE3 deterministic bundle hash with unit test
  (rts-publisher/src/signer.rs tests `sign_bundle_hash_is_deterministic_hex`);
  publisher fails closed when MinIO unconfigured (test `upload_fails_loudly_when_minio_unconfigured`).
  zk-tally Groth16 verification tested in zk-tally/tests/zk_tally_test.rs; bft vote signing
  tested in bft-tally/tests/bft_test.rs.
- **Auth posture**: none of the three axum servers apply authentication middleware today
  (no bearer/SVID checks in rts-api/rts-device-api/rts-ro-api routers). This is the largest
  open gap; eel-auth-rs is declared in the workspace but unused here.
- **Known deferrals**: rts_icr ocr.py returns synthetic fields (TODO PaddleOCR);
  grpc.rs handlers are stubs with no tonic server; sms_fallback base64_decode is hex-only
  (device handlers.rs:195); participation token issuance is stubbed
  (participation_tokens handler); README-documented loadtest/, TLA+ models, and
  declaration HSM ceremony flow were not found in-tree (not verified); dr-failover manual
  steps out of scope here.

## 8. Observability

- tracing_subscriber JSON logs with EnvFilter in every binary main
  (e.g. rts-api main.rs:6-9); structured fields (station_code, idempotency_key, snapshot_id).
- Health endpoint: GET /health on rts-api, rts-device-api, rts-ro-api (plain 200).
- No Prometheus metrics exporter found in rts-svc code (metrics named in README only,
  not verified). Publisher logs anchoring disabled/failed states explicitly
  (scheduler.rs:25, :52).
- Grafana dashboards/alerts listed in README.md "Observability" section are aspirational
  (dashboards not present under services/rts-svc).
