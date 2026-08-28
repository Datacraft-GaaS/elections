# abis-svc

## 1. Overview

The Automated Biometric Identification System (ABIS) provides biometric enrolment,
template storage, 1:N deduplication and 1:1 verification across fingerprint, face and iris
(plus eight extended modalities), human adjudication of ambiguous matches, enclave
attestation of matching workers, drift/fairness monitoring, and a full reports subsystem
(catalog, scheduled runs, signed artifacts, transparency-log anchoring, multi-channel
delivery). Tier T1. Module M03 (WP-3.6) per services/abis-svc/TRACEABILITY.yaml.

- **Runtime**: Rust (axum; crates are members of the root workspace via glob
  `services/*/crates/*` in /Cargo.toml:7 — there is no services/abis-svc/Cargo.toml),
  Python (FastAPI vendor bridge + ONNX/FP workers), Django consoles.
- **Listen ports**: abis-api REST `EEL_ABIS_REST_ADDR` default `0.0.0.0:8080`
  (crates/abis-api/src/main.rs:18-19); health port 8081 documented in main.rs doc comment
  but only one listener is bound in code; deploy/service.yaml declares rest 8080, grpc 8443,
  health 8081, metrics 9090 for abis-api plus separate abis-matcher and abis-adjudicator
  containers.
- **Entrypoint**: crates/abis-api/src/main.rs (`main`, :12). Python entrypoints:
  py/abis-vendor/src/abis_vendor/service.py (FastAPI `app`), py/*/src/*/main.py workers.

## 2. Ownership & Boundaries

- TRACEABILITY.yaml maps FR-M03-001..037/065 to implementations; key anchors:
  matcher pipeline `Pipeline1N`/`Pipeline11` (crates/abis-matcher/src/pipeline.rs),
  adjudication routes, batch dedup, attestation verify.
- **Data ownership**: template index + gallery shards + encrypted blobs (data class A),
  match requests/results (class C), adjudication cases (class B), outbox + audit shadow.
  Actual biometric templates are stored by reference (`probe_ref` / `blob_ref` to MinIO),
  never inline (migrations/V001, V020).
- **Upstream**: enrolment jobs arrive from registration/biometric-edge flows;
  kit attestation events from biometric-edge-svc (eel_kit_attestation_events, V021).
- **Downstream**: transparency-svc (report artifact anchoring, anchor.rs);
  Kafka consumers of `eel.abis.*` topics (docs/EVENTS.md); report deliverers
  (gov-printer adapter, KBC SFTP, email/SMTP).

## 3. Repository Layout

```
services/abis-svc/
  TRACEABILITY.yaml     # FR/AC/EC mapping to code
  data_dictionary.yaml  # Entity definitions
  migrations/V001..V032 # PostgreSQL schema (see section 5)
  crates/
    abis-api/           # Main axum server: routes, middleware (SVID/idempotency/cosign), OpenAPI
    abis-matcher/       # Pipeline1N/11, HNSW index, bloom pruning, modality matchers, PAD, fairness
    abis-attest/        # Enclave attestation: TDX parser, SEV-SNP report, nonce, policy, verifier
    abis-fp-vendor-peak/# Peak-period 3-vendor fingerprint consensus router (HARD-B)
    abis-enrolment/     # Enrolment job domain logic
    abis-adjudication/  # Case/tier/evidence domain
    abis-shielding/ abis-shielded-adjudication abis-smpc-match abis-fhe-match
                        # Template protection, shielded review, MPC matching (V031 tables)
    abis-reports/       # Report definitions, runs, renders (pdf/xlsx/csv/parquet/png/html/json), signer, tlog anchor
    abis-report-publisher/ # Delivery channels: sftp, email, kafka, webhook, gov_printer, kbc_sftp
    abis-reports-adhoc/ # Ad-hoc report authoring API + preview/publish/share/snippets
    abis-render-docx/ abis-render-markdown/
    abis-search-planner/# Costed search plans (estimator/executor)
    abis-watchlist/ abis-twin-registry/ abis-threshold-tuning/ abis-model-canary/
    abis-perf-monitor/ abis-drift-monitor/ abis-cluster-analysis/ abis-rectification/
    abis-standards-*    # ANSI/NIST, EBTS, CBEFF, ICAO, INTERPOL, MINEX record codecs
    abis-modality-*     # ear, gait, latent, palm, periocular, signature, vein, voice
    abis-face-*, abis-fp-*, abis-iris-* # Modality DSP/ML crates
    abis-bulk-ops/ abis-cancelable/ abis-cert-fido/ abis-cert-frvt/ abis-enhancement/
    abis-explainability/ abis-accelerator/ abis-integration-tests/
  tools/report-cli/     # CLI: define/list/run/status/schedule/download/verify/subscribe/deliveries
  py/
    abis-vendor/        # FastAPI vendor bridge (mock/onnx/py adapters) + legacy ABIS worker
    face-onnx-worker/   # ONNX face embedding worker (eel_face_worker)
    fp-dl-worker/       # Deep fingerprint embedder worker (eel_fp_dl_worker)
    iris-onnx-worker/   # Iris embedding worker (eel_iris_worker)
    report-deliverers/  # Python delivery adapters (gov_printer.py, kbc_sftp.py)
  django/
    abis-console/       # Adjudication console (apps/adjudication)
    abis-adhoc-editor/  # Ad-hoc report authoring UI (apps/authoring)
    abis-reports-console/ # Reports console with SpiceDB middleware (middleware/spicedb.py)
  deploy/               # service.yaml, hpa-api/hpa-matcher, networkpolicy, pdb, servicemonitor
  docs/                 # API-CATALOG, ARCHITECTURE, EVENTS, THREAT-MODEL, SLA, SCHEMA-AUDIT...
  loadtest/abis.js      # k6 scenario (referenced by TRACEABILITY AC-M03-01)
  reports-templates/    # Seeded SQL report templates (26 files) + pdf styles
  tests/                # Rust integration tests (enrolment_dedup_flow_test.rs etc.)
```

## 4. Capabilities & Functions

### 4.1 abis-api REST (crates/abis-api/src/lib.rs:35 `app`; handlers cited per file)

| Route | Handler | Notes |
|---|---|---|
| GET /healthz | health::healthz (routes/health.rs:32) | Liveness |
| GET /readyz | health::readyz (health.rs:54) | Readiness |
| GET /openapi.json | openapi::serve_openapi (openapi.rs:98) | utoipa spec |
| POST /v1/enrolment/jobs | create_enrolment_job (routes/enrolment.rs:68) | Create enrolment job |
| GET /v1/enrolment/jobs/{id} | get_enrolment_job (:94) | Status incl. lease/worker |
| PATCH /v1/enrolment/jobs/{id}/withdraw | withdraw_enrolment_job (:122) | Withdraw |
| GET /v1/enrolment/jobs/{id}/quality-report | get_quality_report (:142) | Per-modality quality |
| POST /v1/match/1n | match_1n (routes/match_routes.rs:134) | 1:N identification; outcome mapping via decision_from_outcome (:106) |
| POST /v1/match/11 | match_11 (:214) | 1:1 verification |
| POST /v1/match/batch | match_batch (:306) | Batch dedup submission |
| GET /v1/match/{request_id} | get_match_result (:351) | Result fetch |
| GET /v1/adjudication/cases | list_adjudication (routes/adjudication.rs:70) | Case queue |
| POST /v1/adjudication/cases/{id}/assign | assign_adjudication (:91) | Analyst assignment |
| POST .../decide | decide_adjudication (:118) | Decision recording |
| POST .../escalate | escalate_adjudication (:151) | Tier escalation |
| GET/POST .../evidence | list_evidence (:176) / upload_evidence (:193) | Evidence bundle |
| PATCH /api/v1/abis/adjudication/{case_id} (legacy) | resolve_adjudication (:209) | Back-compat resolve |
| POST /v1/kit/enrolment | kit_enrol (routes/kit.rs:87) | Kit-facing enrolment |
| POST /v1/kit/verify | kit_verify (:120) | Kit-facing verify |
| GET /v1/kit/gallery-config | kit_gallery_config (:151) | Shard config for kits |
| GET /v1/reports/fmr-fnmr | get_fmr_fnmr (routes/reports.rs:95) | Error-rate metrics |
| GET /v1/reports/dedup-summary | get_dedup_summary (:124) | Dedup stats |
| GET /v1/reports/fairness | get_fairness (:150) | Per-group metrics |
| GET /v1/reports/enrolment-throughput | get_enrolment_throughput (:173) | Throughput |
| GET /api/v1/abis/reports (legacy) | get_reports (:188) | Aggregate report |
| POST /api/v1/abis/batch/dedup (legacy) | submit_batch_dedup (routes/batch.rs:10) | Back-compat |
| POST /v1/admin/policies | update_policy (routes/admin.rs:64) | Threshold policy update |
| GET /v1/admin/vendors | list_vendors (:88) | Vendor registry |
| POST /v1/admin/vendors/{id}/health-check | vendor_health_check (:114) | Vendor probe |

### 4.2 Matcher core (crates/abis-matcher)

`Pipeline1N`/`Pipeline11` (src/pipeline.rs); HNSW + bloom-prune + sharded indexes
(src/index/{hnsw,bloom_prune,shard}.rs); modality matchers face/fingerprint/iris +
fusion (src/matcher/); score fusion and ambiguity detection `is_ambiguous`
(src/adjudication.rs); batch runner with offset checkpoints `run_dedup_batch`
(src/batch.rs); FMR/FNMR measurement (src/fmr_fnmr.rs); PAD gate (src/pad/mod.rs).
Tests: tests/matcher_test.rs.

### 4.3 Fingerprint vendor peak (crates/abis-fp-vendor-peak)

Election-day peak mode routing fingerprints through a calibrated three-vendor consensus:
`PeakVendorRouter::match_fingerprints` (src/vendor_router.rs:73) and
`three_way_match` (:86) with 2-of-2 degraded fallback `degraded_match` (:189);
Platt calibration `PlattCalibrator::calibrate` (src/calibration.rs:33);
activation windows `PeakPolicy::is_active` (src/peak_policy.rs:27) and freeze window (:36);
admin API `/v1/admin/peak-policy/enable|disable|status` (src/api.rs:83-88);
Prometheus-style counters (src/metrics.rs:19-27). Tests referenced in crate.

### 4.4 Enclave attestation (crates/abis-attest)

Nonce issuance `AttestationVerifier::issue_nonce` (src/verify.rs:44) and
`AttestationVerifier::verify` (verify.rs:54); policy enforcement
`AttestationPolicy::enforce` with production/development profiles (src/policy.rs:56,66,80);
Intel TDX DCAP parser `TdReport::parse` (src/tdx.rs:30) and
`parse_and_verify_tdx` (tdx.rs:74) against a `TrustedPckStore` trait (:62; DevPckStore
trusts all chains — dev only); AMD SEV-SNP `SevSnpReport::parse` (src/sev.rs:48);
profile selection via env `EEL_ABIS_ATTESTATION` (attestation.rs:56). Verification rows
persist to enclave_report_verifications (migrations/V015).

### 4.5 Reports subsystem (crates/abis-reports, -adhoc, -publisher)

- Catalog/runs API `/v1/reports/*` (abis-reports/src/api.rs:276-289): definitions CRUD,
  schedule_preview, create_run, artifacts, `verify_artifact` (inclusion check),
  deliveries.
- Rendering pipeline src/render/{pdf,xlsx,csv,parquet,png,html,json}.rs; extraction
  src/extractor.rs; scheduler src/scheduler.rs (background run loop).
- Signing: `sign_artifact` (src/signer.rs:25) posts JCS envelope to HSM at
  `HSM_SIGNER_URL` (/v1/sign); fail-closed via `HSM_SIGN_FAIL_CLOSED`, else ephemeral
  Ed25519 dev fallback. Anchoring: src/anchor.rs appends artifact hash to transparency log
  (`EEL_TLOG_URL`-style env at anchor.rs:41).
- Delivery channels (abis-report-publisher/src/channels/): sftp.rs (key-based auth,
  EEL_SFTP_PRIVKEY_PATH :31, EEL_SFTP_USER :118), email.rs (SMTP env :32-96), kafka.rs
  (EEL_KAFKA_BROKERS :18), webhook.rs, gov_printer.rs (EEL_GOV_PRINTER_ADAPTER_URL :14),
  kbc_sftp.rs (:14); orchestrated by pipeline.rs with metrics.rs counters.
- Ad-hoc reports API (abis-reports-adhoc/src/api.rs:152-162): reports CRUD, versions,
  preview, publish, shares grant/revoke, snippets. Preview expiry enforced by
  eel_adhoc_previews.expires_at (V029).

### 4.6 Python workers (py/)

- abis-vendor FastAPI (src/abis_vendor/service.py): POST /extract/{modality},
  POST /match/1x1, POST /match/1xN, GET /health; adapter selected by `EEL_VENDOR_MODE`
  (service.py lifespan: mock / onnx requiring EEL_ONNX_MODEL_PATH / py-fingerprint).
  `AbisVendorWorker.process_one/run_batch` (worker.py:17, :30; max 10 parallel) calls the
  legacy ABIS via LegacyAbisAdapter (adapter.py).
- face-onnx-worker, fp-dl-worker, iris-onnx-worker: FastAPI /embed,/match,/health
  (e.g. fp-dl-worker main.py:70,:81,:95); ONNX model path via EEL_FP_DL_ONNX_PATH in
  the Rust-side loader (crates/abis-fp-dl-onnx/src/embed.rs:95).
- report-deliverers: python mirrors of gov-printer/KBC SFTP adapters with tests.

### 4.7 CLI (tools/report-cli)

Subcommands (src/commands/): define, list, run, status, schedule, download, verify,
subscribe, deliveries (main.rs dispatch); api_client.rs talks to /v1/reports.

## 5. Data Model

Key tables across migrations V001-V032 (all prefixed eel_ unless noted):

| Table (migration) | Key columns | Purpose |
|---|---|---|
| eel_template_index (V001) | index_id, voter_id, modality, template_hash, probe_ref, deleted_at | Template metadata only; blob by reference |
| eel_match_requests (V002) | request_id, status queued/processing/complete/failed, idempotency_key UNIQUE, caller_id | Match tracking |
| eel_match_results (V003) | outcome match/no_match/adjudicate/insufficient_quality, confidence_score, attestation_hash | Results |
| eel_adjudication_cases (V004) | sla_deadline_at default +24h, decision enum | v1 cases |
| eel_enclave_attestations (V005) | profile software/tdx/sev_snp, attestation_hash | Worker attestations |
| eel_drift_alerts (V006) | drift_type fmr/fnmr/per_group, demographic_group, model_card_version | Drift monitoring |
| eel_abis_outbox (V007) | event_id UNIQUE, topic, retry_count | Transactional outbox |
| eel_abis_audit_shadow (V008) | operation, actor_id, resource_id | Audit trail |
| eel_enrolment_jobs (V009) | subject_id, status Received->..., lease_expires_at, worker_id | Enrolment workflow |
| eel_enrolment_decisions (V010) | gallery_shard_id, merge_from, adjudicator_id | Dedup decisions |
| eel_quality_reports (V011) | per_modality JSONB, retake_reasons | Capture quality |
| eel_adjudication_cases_v2 (V012) | tier Tier1Analyst..., state, evidence_bundle_ref, decision_type | Current case model |
| eel_reviewer_actions (V013) | public_key_hex, prev_action_hash, canonical_json | Hash-chained reviewer actions |
| eel_panel_votes (V014) | vote Approve/Reject/Abstain, public_key_hex | Panel decisions |
| enclave_report_verifications (V015) | nonce_hex, mrtd_hex, measurement_hex, verdict | TDX/SEV verification records |
| eel_shards, eel_gallery_templates, eel_hnsw_snapshots, eel_demographic_cohorts, eel_match_candidates (V020) | kek_version, blob_ref, shard status | Sharded gallery |
| eel_kits, eel_kit_attestation_events, eel_vendor_registry (V021) | verdict, health | Kits & vendors |
| eel_kek_generations, eel_template_kek_bindings, eel_blob_manifest, eel_pad_events (V022) | retention, attack_type | Crypto/PII lifecycle, PAD events |
| eel_reviewers, eel_reviewer_specialisations, eel_idempotency_keys (V023) | key_hash, request_body_hash | Reviewer roster, idempotency store |
| eel_fairness_evaluations, eel_legacy_kiems_migration, eel_model_cards (V024) | breach flags | Fairness, migration, model governance |
| V025 | monthly partitions + TTL triggers for audit shadow | Partitioning |
| V026 | correlation id indexes on requests/jobs/cases | Cross-cutting correlation |
| eel_report_definitions/-runs/-artifacts/-tlog_anchors/-subscribers/-subscriptions/-deliveries/-access_log (V027) | slug, status, next_attempt | Reports catalog; access log stores CIDR-truncated IPs |
| V028 | seed rows for report catalog | Seed data |
| eel_adhoc_reports/-versions/-previews/-shares/-snippets (V029) | visibility, expires_at | Ad-hoc authoring |
| eel_palm/latent/voice/vein/signature_templates, eel_watchlists/-entries (V030) | case_ref for latents | Extended modalities |
| eel_bio_protection_schemes, eel_template_protections, eel_fhe_contexts, eel_smpc_sessions (V031) | scheme params | Cancelable templates, FHE/MPC |
| eel_threshold_profiles, eel_model_canaries, eel_duplicate_clusters, eel_twin_registry, eel_rectification_requests, eel_bulk_operations, eel_explainability_records (V032) | active flags, SLA | Ops excellence |

Kafka topics produced via outbox (docs/EVENTS.md, ground truth): `eel.abis.match.v1`
(match_requested.v1, match_completed.v1), `eel.abis.adjudication.v1` (adjudication_opened.v1,
adjudication_decided.v1), `eel.abis.enclave.v1` (enclave_attested.v1),
`eel.abis.drift.v1` (drift_alert.v1, unsigned). All CloudEvents v1.0, Ed25519-signed except
drift_alert; probe_ref/candidate_ref deliberately excluded from broker payloads (class A).

## 6. Operation

```bash
cargo test -p abis-api
cargo test -p abis-matcher
cargo test -p abis-attest
cargo test -p abis-fp-vendor-peak
cargo run -p abis-api                       # REST :8080
cd py/abis-vendor && pip install -e . && uvicorn abis_vendor.service:app
cd django/abis-console && python manage.py test
k6 run loadtest/abis.js
tools/report-cli --help                     # cargo run -p report-cli --
```

Env vars (grep verified): EEL_ABIS_REST_ADDR (main.rs:18), EEL_ABIS_SPIRE_REQUIRED
(middleware.rs:17), EEL_ABIS_DATABASE_URL / DATABASE_URL (state.rs:17-18),
EEL_ABIS_ATTESTATION (attestation.rs:56), EEL_FP_DL_ONNX_PATH (embed.rs:95),
EEL_VENDOR_MODE + EEL_ONNX_MODEL_PATH (service.py), EEL_SMTP_HOST/PORT/FROM/USER/PASS
(email.rs), EEL_GOV_PRINTER_ADAPTER_URL (gov_printer.rs:14),
EEL_KBC_SFTP_ADAPTER_URL (kbc_sftp.rs:14), EEL_KAFKA_BROKERS (channels/kafka.rs:18),
EEL_SFTP_PRIVKEY_PATH / EEL_SFTP_USER (sftp.rs), HSM_SIGNER_URL + HSM_SIGN_FAIL_CLOSED
(reports/signer.rs:35-36), tlog URL env in anchor.rs:41.

Deploy: deploy/service.yaml (abis-api with rest/grpc/health/metrics ports, abis-matcher,
abis-adjudicator), hpa-api.yaml + hpa-matcher.yaml autoscaling, pdb.yaml,
networkpolicy.yaml, servicemonitor.yaml (Prometheus Operator).

Degradation: lazy DB pool keeps router buildable without Postgres; store failures surface
as 503 problem+json (state.rs doc comment). Attestation profiles degrade software <->
tdx/sev_snp via policy (policy.rs:80). Peak-mode degradation path two_of_two/single_vendor
(degrade.rs:38,49). Vendor failover covered by tests/vendor_failover_test.rs.

## 7. Safeguards & Security

- **SPIRE SVID auth**: spire_auth middleware checks X-SVID-Token when
  `EEL_ABIS_SPIRE_REQUIRED=true`, rejects literal "invalid" token for tests
  (middleware.rs:15-47). Dev bypass otherwise. Note: middleware functions exist but are not
  layered onto the router in lib.rs `app()` — only cosign_response is applied (lib.rs:103).
  Wiring gap (deferral, matches TRACEABILITY FR-M03-036 "SpiceDB fail-closed pending").
- **Idempotency**: require_idempotency_key middleware demands Idempotency-Key on mutating
  methods (middleware.rs:53-78); durable replay store lookup/record with SHA-256 key hashes
  and 409 on fingerprint mismatch (idempotency.rs:16-46 lookup, :49 record; table
  eel_idempotency_keys V023). EC-DAT-01 enforcement point.
- **Response signing**: cosign_response adds X-EEL-Signature header (middleware.rs:84-92) —
  stub value "stub-cosign-v1" (deferral; real signer pending eel-api-server-rs integration).
- **Template protection**: templates encrypted by reference with KEK versioning
  (eel_gallery_templates.kek_version, V020; KEK generations/bindings V022);
  cancelable-template schemes and FHE/SMPC contexts (V031); SMPC dot-product protocol
  (abis-smpc-match/src/protocol.rs) and session handling (session.rs). Enclave boundary:
  match results carry attestation_hash linking result -> verified enclave
  (EVENTS.md match_completed.v1; table V003).
- **HSM signing fail-closed option**: reports/signer.rs:35-45 honors
  HSM_SIGN_FAIL_CLOSED; falls back to ephemeral Ed25519 only in dev with warning.
- **Audit logging**: every match/adjudication/batch op written to eel_abis_audit_shadow
  (V008) with partitioning + TTL (V025); hash-chained reviewer actions (V013
  prev_action_hash + canonical_json); report access log with IP truncation to /24 //48
  (reports/api.rs `cidr_prefix`, :296).
- **Input validation**: sqlx parameterised queries (TRACEABILITY FR-M03-035); modality
  route/body mismatch returns 422 (py service.py extract); image decode guards in workers;
  MRZ/quality gates in matcher (quality_gate stub noted FR-M03-008).
- **Error surfaces**: problem+json style errors; store failures -> 503; conflict -> 409
  (ApiError::Conflict idempotency.rs:36).
- **Tests verifying safeguards**: crates/abis-api/tests/api_test.rs,
  crates/abis-attest/tests/attest_test.rs (FR-M03-011), crates/abis-matcher/tests/
  matcher_test.rs (fmr_fnmr_mock, batch_dedup_empty), services/abis-svc/tests/
  {integration_test,enrolment_dedup_flow_test,attestation_replay_test,vendor_failover_test,
  fairness_drill_test,adjudication_workflow_test}.rs, py/abis-vendor/tests/test_service.py,
  tools/report-cli/tests/cli_test.rs.
- **Known deferrals** (from TRACEABILITY notes + code): NFIQ2 quality gate stub
  (FR-M03-008); ISO 19794 template compliance deferred to vendor SDK (FR-M03-009);
  SpiceDB biometric-auth for adjudication pending (FR-M03-018/:019); cosign CI check
  pending (FR-M03-014); audit-to-tlog pending (FR-M03-015); loadtest scenario marked
  scaffold (AC-M03-01). DevPckStore trusts all PCK chains (tdx.rs:62-71) — production
  store required before TDX enforcement is meaningful.

## 8. Observability

- Health endpoints: GET /healthz and GET /readyz with DB-touching readiness
  (routes/health.rs:32,54); dedicated health port 8081 declared in deploy/service.yaml.
- Metrics: ServiceMonitor scrape of port 9090 (deploy/servicemonitor.yaml); in-process
  counters in abis-fp-vendor-peak/src/metrics.rs and abis-report-publisher/src/metrics.rs.
- Tracing: tracing_subscriber in abis-api main.rs:13; structured logs in publisher channels.
- Dashboards/alerts referenced by docs/OPERATIONS.md; drift alerts double as an alerting
  feed via eel.abis.drift.v1 (docs/EVENTS.md consumers: model-ops dashboard, on-call).
