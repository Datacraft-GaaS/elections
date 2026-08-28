# ABIS Platform Core and Operations Crates

Reference documentation for the core matching/lifecycle and operations/governance
crates under `services/abis-svc/crates/`. All line citations refer to the
current tree. Crate names below drop the `eel-` package prefix where the
directory name differs (e.g. directory `abis-vendor`, package `eel-abis-vendor`).

## Environment prerequisite

Crates that link `oqs` (declared once in the workspace root,
`Cargo.toml:96`) require an OpenSSL toolchain on macOS. From workspace
testing, builds need `OPENSSL_DIR` pointed at the OpenSSL prefix plus a
`RUSTFLAGS` entry of the form `-L <prefix>/lib` so `oqs-sys` can find
`libcrypto`. Without both, linking fails even though the pure-Rust crates
compile cleanly.

---

# Core matching and pipeline

## abis-matcher

The largest crate (~3,500 lines) and the orchestration heart of the service.
Crate-level docs (`src/lib.rs:1-12`) describe a multi-modal matching engine for
fingerprint (10-print, ISO/IEC 19794-2), face (512-d embedding), and iris
(2048-bit IrisCode) at 30M-enrolment scale.

### Module tree

- `domain.rs` -- core value types shared by everything else
- `modality/` -- template types and the `Template` trait
- `matcher/` -- per-modality matchers and score fusion
- `index/` -- HNSW ANN index, shard hasher, Bloom pruner
- `quality/`, `fairness/`, `pad/` -- quality gates (NFIQ2 / ICAO /
  ISO 29794), per-demographic FMR/FNMR slices (ISO/IEC 19795-10), and the
  presentation attack detection interface
- `pipeline.rs` -- the `MatchPipeline` trait + wrappers
- `adjudication.rs`, `batch.rs`, `fmr_fnmr.rs` -- case management, dedup
  runs, accuracy harness; `mock_matcher.rs` -- deterministic mock behind
  the `mock` feature / tests

### Pipeline abstraction

- `MatchPipeline` trait (`pipeline.rs:5`): `match_1n(probe_ref,
  &[TemplateIndex], threshold)` and `match_11(probe_ref, gallery_ref,
  threshold)`, both returning `MatchResult`. This is the seam every consumer
  programs against; production matchers and the mock both implement it.
- `Pipeline1N` / `Pipeline11` (`pipeline.rs:11,26`): thin owning wrappers
  around `Box<dyn MatchPipeline>` exposing `run()`.

### Domain model

Defined in `domain.rs`: `TemplateIndex` (`:4`, template reference + hash),
`MatchRequest` (`:12`), `MatchResult` (`:21`) with `MatchScore` /
`ModalityScore` (`:30`, `:37`), `MatchOutcome` (`:44`), plus early stubs kept
for compatibility: `AdjudicationCase`/`Status`/`Decision` (`:52,65,73`),
`EnclaveAttestation`/`AttestationProfile` (`:82,92`), `DriftAlert` (`:99`).
The adjudication stub is explicitly superseded by the `abis-adjudication`
crate (see its lib.rs note).

### Index structures

- `TemplateIndex` trait (`index/mod.rs:31`); candidate record `Candidate`
  (`index/mod.rs:15`).
- HNSW ANN index (`index/hnsw.rs`): documented parameters M=32,
  ef_construction=200, ef_search=100 (`:3`, `EF_SEARCH` const `:16`);
  targets 512-d face embeddings at 30M scale when sharded. Struct
  `HnswIndex` at `:62`, constructor `new(dim)` at `:275`, `TemplateIndex`
  impl at `:283`.
- Sharding (`index/shard.rs`): `shard_id()` (`:10`) maps a subject to one of
  256 shards via `blake3(subject_id)[0] mod 256`; `shard_distribution()`
  (`:18`) reports balance across shards.
- Demographic Bloom pruner (`index/bloom_prune.rs`): one Bloom filter per
  (county, sex, age-bucket) slice, 8 bits-per-key, 0.01 FPR, double hashing
  with FNV1a+FNV1b (`:3-7`). `DemographicBloom` at `:124` with `insert` /
  `may_contain` / `candidate_slices` (`:139`, `:149`, `:157`).

### Matchers and fusion

- `Matcher<T>` trait and `Score`: `matcher/mod.rs:37` and `:17`.
- Face: cosine similarity over L2-normalised 512-d embeddings --
  `FaceMatcher` (`matcher/face_matcher.rs:25`), `cosine_similarity` (`:57`),
  optional cohort normalisation stats `CohortStats` (`:8`).
- Fingerprint: Bozorth3-style minutia matcher
  (`matcher/fingerprint_matcher.rs`; algorithm overview `:3-8`, ANSI 378
  tolerances +/-40 px / +/-15 deg / +/-2 ridge counts `:10-13`),
  `raw_score` at `:57`, normalised against a reference score of 40.
- Iris: masked Hamming distance on 2048-bit IrisCodes
  (`matcher/iris_matcher.rs:8`, threshold constructor `:25`).
- Fusion: `FusionPolicy` weights per modality (`matcher/fusion.rs:7`),
  `fuse()` weighted sum producing a `FusionResult` (`:62`).

### Modality templates

- `Template` trait (`modality/mod.rs:21`); `Modality` enum `:14`.
- Face: `FaceTemplate` (`modality/face.rs:50`) with `IcaoFlags` conformance
  (`:7`, scoring `:16`, `all_pass` `:26`) and optional landmarks.
- Fingerprint: ISO/IEC 19794-2 round-trip -- `parse_iso19794_2`
  (`modality/fingerprint.rs:157`) and `serialise_iso19794_2` (`:264`);
  minutia model `Minutia` (`:67`), `FingerPosition` (`:12`),
  NFIQ2-style `quality_from_minutiae` (`:315`).
- Iris: `IrisTemplate` (`modality/iris.rs:16`, 2048-bit constant `:6`),
  masked Hamming with minimum-overlap guard (`hamming_distance`, `:54`).

### Quality, fairness, PAD

- Quality gates: `QualityPolicy` (`quality/mod.rs:9`) and per-modality
  dispositions `gate_fingerprint` (`:46`), `gate_face` (`:57`),
  `gate_iris` (`:68`), `gate_finger_count` (`:79`).
- Fairness harness: `fairness/mod.rs` extends `fmr_fnmr.rs` with demographic
  disaggregation; documented tolerances are <=5% variation across sex and
  <=10% across age buckets (`fairness/mod.rs:4`). `FairnessSlice` (`:11`),
  `FairnessReport::compute` (`:53`), equalised-odds `detect_breaches`
  (`:123`).
- PAD: `PadDetector` trait (`pad/mod.rs:47`), `AttackType` (`:10`),
  `PadResult` (`:27`). The production path wires a vendor SDK; the in-crate
  `SoftwarePad` (`pad/mod.rs:54`) always returns `Live` -- development/test
  only.

### Accuracy, batch, and adjudication helpers

- FMR/FNMR harness: `compute_fmr_fnmr` (`fmr_fnmr.rs:17`) drives any
  `MatchPipeline` over labelled genuine/impostor pairs and returns
  `FmrFnmrHarness` (`:6`) with `fmr`, `fnmr`, pair totals. FNMR = missed
  genuine / total genuine; FMR = accepted impostor / total impostor.
- Batch dedup: `BatchDedupRun` (`batch.rs:7`),
  `new_batch_job(election_id, mode)` (`:13`), `run_dedup_batch` (`:23`).
- Adjudication stubs: `open_adjudication_case` (`adjudication.rs:5`),
  `record_decision` (`:21`), ambiguity banding `is_ambiguous(score, lower,
  upper)` (`:28`) -- the stubs the full `abis-adjudication` crate grew out
  of.

### MockMatcher

`MockMatcher` (`mock_matcher.rs:8`, exported only under
`#[cfg(any(test, feature = "mock"))]`, `lib.rs:26-33`) implements
`MatchPipeline` (`mock_matcher.rs:47`) using SHA-256-derived XOR similarity:
identical strings score 1.0, random pairs ~0.5 (`xor_similarity`,
`mock_matcher.rs:18`). The `mock` feature is on by default
(`Cargo.toml`: `default = ["mock"]`).

### Public API (selected)

| Item | Location |
|---|---|
| `MatchPipeline`, `Pipeline1N`, `Pipeline11` | `src/pipeline.rs:5,11,26` |
| `MockMatcher` (feature `mock`) | `src/mock_matcher.rs:8` |
| `HnswIndex`, `shard_id`, `DemographicBloom` | `src/index/hnsw.rs:62`, `src/index/shard.rs:10`, `src/index/bloom_prune.rs:124` |
| `FaceMatcher`, `FingerprintMatcher`, `IrisMatcher`, `fuse` | `src/matcher/*.rs` |
| `FaceTemplate`, `FingerprintTemplate`, `IrisTemplate` | `src/modality/*.rs` |
| `gate_fingerprint/face/iris/finger_count` | `src/quality/mod.rs:46,57,68,79` |
| `FairnessReport::compute/detect_breaches` | `src/fairness/mod.rs:53,123` |
| `compute_fmr_fnmr` | `src/fmr_fnmr.rs:17` |
| `run_dedup_batch` | `src/batch.rs:23` |

### Verification

54 integration-style tests in `tests/matcher_test.rs` (1,019 lines), 2 inline
tests in `src/index/shard.rs`, plus a criterion benchmark
(`benches/matching_bench.rs`, gated behind the `bench` feature). Consumers
found in workspace manifests: `abis-api` (with the `mock` feature),
`abis-fp-consensus`, `abis-fp-mcc`, `abis-fp-level3`, `abis-fp-ridge-count`,
and `abis-fp-cdeffs`.

## abis-vendor

Vendor SDK abstraction (`src/lib.rs:1-9`): a unified `VendorAdapter` trait over
Neurotechnology / Innovatrics / NEC / Idemia-class SDKs.

Key components:
- `VendorAdapter` trait and shared image/template/score types:
  `src/adapter.rs:123` (trait), `Template` enum `:72`,
  `VendorCapabilities` `:99`, `HealthStatus` `:111`.
- `MockVendor`: deterministic test vendor, `src/mock.rs:15` (impl `:50`).
- `PythonVendor`: HTTP bridge to the `py/abis-vendor` sidecar,
  `src/python_bridge.rs:82` (`from_env` `:89`, impl `:167`).
- `GrpcVendor`: gRPC sidecar bridge, currently a stub,
  `src/grpc_bridge.rs:23` (impl `:40`).
- `VendorRouter`: per-modality routing with health-based failover,
  `src/router.rs:34` (`with_fallback` `:53`, `VendorAdapter` impl `:127`).

Public API: re-exported from `src/lib.rs:14-23`. Verification: 1 external
test file (`tests/vendor_test.rs`) with a single test -- thin relative to the
surface area. Plugs into: consumed by `abis-api` and `abis-fp-vendor-peak`.

## abis-fp-vendor-peak

Cross-vendor 2-of-3 fingerprint consensus during the election freeze window
T-1..T+7 (`src/lib.rs:1-17`). Wraps the vendor router with a three-way
dispatch (internal matcher + vendor A + vendor B), fused via MinAgreementPolicy
from `eel-abis-fp-consensus`, with per-vendor Platt scaling so heterogeneous
scores are comparable before voting.

Key components:
- `PeakPolicy` (activation window): `src/peak_policy.rs:13`, `is_active`
  `:27`, `freeze_window` `:36`.
- Calibration: `PlattCalibrator` `src/calibration.rs:18` (`calibrate` `:33`),
  `VendorCalibrators::calibrate_all` `:74`.
- Routing: `PeakVendorRouter` `src/vendor_router.rs:54`, `three_way_match`
  `:86`, degraded-mode `degraded_match` `:189`; `VendorTriple` `:34`,
  `VendorMatcher` trait `:26`; `DegradeOutcome`/`DegradedCandidate`
  `src/degrade.rs:12,23` (2-of-2 and single-vendor constructors `:38,:49`).
- Axum sub-router for policy control: `api.rs` (`router()` `:83`,
  `PolicyState` `:19`); metrics counters and `snapshot()`
  `src/metrics.rs:19-51`; `PeakError` variants including `ConsensusFailed`
  and `InsufficientVendors` at `src/lib.rs:38-52`.

Public API table: re-exports at `src/lib.rs:19-24`. Verification: 22
external + 14 inline tests. Plugs into: no in-repo consumer yet; it is a
workspace member called out as a HARD-B deliverable in the root `Cargo.toml`
comments.

## abis-fusion-advanced

Advanced multi-modal fusion beyond `abis-matcher/src/matcher/fusion.rs`:
score-level, feature-level, and decision-level fusion plus cohort and
spatiotemporal context filtering. Notable: this crate has **no crate-level
doc header** in `src/lib.rs` -- the module list is self-describing only.

Key components:
- Score level: min-max / z-score / tanh / double-sigmoid normalisation,
  t-norm/s-norm, weighted sum, product and likelihood-ratio fusion
  (`src/score_level.rs:3-95`).
- Feature level: `concatenate` (`src/feature_level.rs:3`); `pca_reduce` is
  explicitly a stub ("wire real PCA decomposition in production", `:8-10`);
  `KernelPca` trait `:21`. Decision level: `majority_vote` `:3`,
  `weighted_vote` `:11`, `borda_count` `:28`, `bayesian_belief` `:47`
  (`src/decision_level.rs`).
- Cohort normalisation: `CohortStats` / `CohortNormalizer`
  (`src/cohort.rs:6,13`). Context: `SpatiotemporalFilter` over location/time
  windows (`src/context.rs:18`, `filter` `:31`). `FusionError`:
  `src/error.rs:4`.

Verification: 22 external tests. Plugs into: consumed by
`abis-fp-consensus`.

---

# Lifecycle

## abis-enrolment

Enrolment lifecycle and 1:N deduplication pipeline (`src/lib.rs:1-4`): jobs
move from Received through quality check, dedup, adjudication routing, and
gallery placement.

Key components:
- Domain: `EnrolmentJob` (`src/domain.rs:122`), `EnrolmentStatus` (`:29`),
  `EnrolmentSource` (`:9`), `CapturedTemplate` (`:85`), `QualityReport`
  (`:105`, acceptability `:115`), `PadResult` (`:97`), `DedupDecision` /
  `DedupResult` / `ProbableMatch`.
- Pure pipeline functions (`src/pipeline.rs`): `ingest` (`:32`),
  `apply_quality_report` (`:44`), `apply_dedup_result` (`:71`),
  `route_decision` (`:82`), `finalise` (`:119`); mutable `PipelineState`
  (`:15`). Event builders (CloudEvents-style): received / rejected /
  enrolled / dedup-hold / subject-finalised
  (`src/events.rs:18,36,53,70,87`).
- Async workers: `start_workers` + `OrchestratorConfig`
  (`src/orchestrator.rs:41,17`); persistence: sqlx `JobRow`
  (`src/repository.rs:139`).

Public API: re-exports at `src/lib.rs:11-17`. Verification: 18 external tests
(`tests/enrolment_test.rs`). Plugs in as a standalone library today -- no
other crate's `Cargo.toml` depends on it; its logic is mirrored (not yet
replaced) by the API layer.

## abis-adjudication

Full adjudication workflow, FSM, and reviewer audit chain
(`src/lib.rs:1-4`). The lib docs state it was expanded from the stub in
`abis-matcher/src/adjudication.rs`, which is deliberately left in place --
this crate is the real implementation.

Key components:
- Domain: `AdjudicationCase` (`src/domain.rs:76`, ctor `:94`), SLA policy
  `AdjudicationSlaPolicy` (`:47`, deadline `:64`, breach checks `:69,121`),
  `ReviewerAction` (`:134`), `PanelVote` (`:181`), `Reviewer` tiers (`:201`).
- FSM: `transition` (`src/fsm.rs:38`), `panel_quorum_reached` (`:102`),
  `TransitionContext` (`:114`), `FsmError` (`:22`). Assignment: tier-aware
  reviewer selection `select_reviewer` (`src/assignment.rs:19`),
  `Assignment` (`:56`).
- Audit chain: `create_action` (`src/audit.rs:126`), canonical hashing
  (`:73`), signature verification (`verify_action_signature` `:82`,
  `verify_chain` `:107`) behind an `HsmSigner` trait (`:16`). Events: case
  opened / assigned / decided / escalated (`src/events.rs:15,38,58,79`);
  persistence: sqlx `CaseRow` (`src/repository.rs:103`).

Public API: re-exports at `src/lib.rs:12-20`. Verification: 31 external
tests, including Ed25519 test-signer coverage of the audit chain. Plugs into:
consumed by `abis-shielded-adjudication` (with the `test-utils` feature).

## abis-api

The HTTP service (`src/lib.rs:1`). Binary entrypoint `src/main.rs` binds
`EEL_ABIS_REST_ADDR` (default `0.0.0.0:8080`).

### Router: every route from `app()`

`app(state)` is defined at `src/lib.rs:37`. Nested groups under `/v1`
(`src/lib.rs:80-86`) plus legacy `/api/v1/abis` (`:88-96`) and top-level
health/openapi (`:99-104`):

| Path | Method | Handler |
|---|---|---|
| `/v1/enrolment/jobs` | POST | `routes::enrolment::create_enrolment_job` |
| `/v1/enrolment/jobs/{id}` | GET | `get_enrolment_job` |
| `/v1/enrolment/jobs/{id}/withdraw` | PATCH | `withdraw_enrolment_job` |
| `/v1/enrolment/jobs/{id}/quality-report` | GET | `get_quality_report` |
| `/v1/match/1n` | POST | `match_routes::match_1n` |
| `/v1/match/11` | POST | `match_11` |
| `/v1/match/batch` | POST | `match_batch` |
| `/v1/match/{request_id}` | GET | `get_match_result` |
| `/v1/adjudication/cases` | GET | `adjudication::list_adjudication` |
| `/v1/adjudication/cases/{id}/assign` | POST | `assign_adjudication` |
| `/v1/adjudication/cases/{id}/decide` | POST | `decide_adjudication` |
| `/v1/adjudication/cases/{id}/escalate` | POST | `escalate_adjudication` |
| `/v1/adjudication/cases/{id}/evidence` | GET/POST | `list_evidence` / `upload_evidence` |
| `/v1/kit/enrolment` | POST | `kit::kit_enrol` |
| `/v1/kit/verify` | POST | `kit_verify` |
| `/v1/kit/gallery-config` | GET | `kit_gallery_config` |
| `/v1/reports/fmr-fnmr` | GET | `reports::get_fmr_fnmr` |
| `/v1/reports/dedup-summary` | GET | `get_dedup_summary` |
| `/v1/reports/fairness` | GET | `get_fairness` |
| `/v1/reports/enrolment-throughput` | GET | `get_enrolment_throughput` |
| `/v1/admin/policies` | POST | `admin::update_policy` |
| `/v1/admin/vendors` | GET | `list_vendors` |
| `/v1/admin/vendors/{id}/health-check` | POST | `vendor_health_check` |
| `/api/v1/abis/match/1n` | POST | `match_1n` (legacy alias) |
| `/api/v1/abis/match/11` | POST | `match_11` (legacy alias) |
| `/api/v1/abis/match/{request_id}` | GET | `get_match_result` (legacy alias) |
| `/api/v1/abis/adjudication` | GET | `list_adjudication` (legacy) |
| `/api/v1/abis/adjudication/{case_id}` | PATCH | `resolve_adjudication` (legacy) |
| `/api/v1/abis/batch/dedup` | POST | `batch::submit_batch_dedup` (legacy) |
| `/api/v1/abis/reports` | GET | `get_reports` (legacy) |
| `/healthz` | GET | `health::healthz` |
| `/readyz` | GET | `health::readyz` |
| `/openapi.json` | GET | `openapi::serve_openapi` |

OpenAPI is generated with utoipa (`src/openapi.rs`); the module doc also
mentions a Swagger UI route, but only `/openapi.json` is actually mounted.

### Auth posture: `spire_auth`

Defined at `src/middleware.rs:102`. Semantics when
`EEL_ABIS_SPIRE_REQUIRED=true`:
1. SPIFFE identity recovered from Envoy-propagated mTLS headers -- XFCC
   (`By=` / `URI=` fields) or direct `X-Client-Spiffe-Id`
   (`extract_spiffe`, `src/middleware.rs:50`).
2. Must parse as `spiffe://<trust-domain>/ns/<ns>/sa/<sa>` (`SpiffeIdentity::parse`, `:25`)
   with trust domain exactly `eel.cluster` (`:111`).
3. If `EEL_ABIS_ALLOWED_NAMESPACES` is set (comma-separated), only those
   namespaces are admitted; default posture admits enrolment /
   adjudication / admin namespaces (`:114-120`).

Leaving `EEL_ABIS_SPIRE_REQUIRED` unset/false is an explicit dev bypass
(`:100-101`). Wiring gap: `spire_auth` and `require_idempotency_key`
(`src/middleware.rs:134`, requires `Idempotency-Key` on mutating methods)
are **defined but not applied in `app()`** -- the only middleware layered on
the router is `cosign_response` (`src/lib.rs:105`). Per-route idempotency is
instead enforced inside the match handlers via `idempotency::lookup` /
`record` (e.g. `src/routes/match_routes.rs:167,196-197,232,296`).

### Idempotency support (ADR-0011)

`src/idempotency.rs`: mutating operations deduplicate on
`(operation, Idempotency-Key)` in the `eel_idempotency_keys` table. Keys are
stored as SHA-256 hashes scoped by operation (`key_hash`, `:15`); first
accepted response is snapshotted and replayed verbatim (`lookup` `:29`,
`replay_response` `:106`); key reuse with a different body fingerprint is a
409 (`:47-51`); `fingerprint_request` SHA-256s exact request bytes (`:122`).

### Errors, state, and the store.rs question

RFC 9457 problem+json: `ApiError` (`src/error.rs:11`) maps Validation /
NotFound / Conflict / Store(sqlx) to 400/404/409/503 and emits
`application/problem+json` bodies with `type`, `title`, `status`, `detail`
(`src/error.rs:51-63`). Store internals never leak (`:28-29,44-49`).

`AppState` holds a lazily-connected sqlx `PgPool` (`src/state.rs:14-33`;
`EEL_ABIS_DATABASE_URL`, falling back to `DATABASE_URL`, then
`postgres://127.0.0.1:5432/eel_abis`). Construction never touches the
network; failures surface as 503 problem+json.

Note on `store.rs`: it does **not exist** -- `src/` contains only `error.rs`,
`idempotency.rs`, `lib.rs`, `main.rs`, `middleware.rs`, `models.rs`,
`openapi.rs`, `routes/`, `state.rs`. There is no undeclared module to flag.

Verification: 5 inline tests (3 in `idempotency.rs`, others in route
modules); no `tests/` directory in-crate. Cross-crate HTTP scenarios live in
the `abis-integration-tests` harness instead.

### Honest status notes

- `cosign_response` sets a placeholder `x-eel-signature: stub-cosign-v1`
  header on every response (`src/middleware.rs:165-173`).
- Several handlers return static data: all four enrolment endpoints ignore
  their inputs (`create_enrolment_job` fabricates a UUID job,
  `get_enrolment_job` always answers `"subject-001" / "processing"`,
  `src/routes/enrolment.rs:68-136`); reports endpoints return fixed numbers
  regardless of query (`fmr: 0.001, fnmr: 0.005`,
  `src/routes/reports.rs:105-113`); `match_11` matches through `MockMatcher`
  (`src/routes/match_routes.rs:237`).
- `main.rs` documents a `EEL_ABIS_HEALTH_ADDR` port but never binds one;
  health endpoints share the REST listener.
- Consumes `abis-matcher` (mock feature), `abis-attest`, and
  `eel-abis-vendor`; nothing depends on it.

## abis-integration-tests

No `src/` -- it is a harness-only crate whose `Cargo.toml` wires the
cross-crate HTTP tests living at `services/abis-svc/tests/` (previously they
had no manifest and never compiled). Five `[[test]]` targets:

| Test | Scenario |
|---|---|
| `enrolment_dedup_flow_test` | Full pipeline: ingest -> quality -> dedup -> route -> enrol; asserts decision + event emission; clean-path and duplicate cases |
| `adjudication_workflow_test` | Case created -> assigned -> decided -> tlog audit entry present |
| `fairness_drill_test` | Synthetic unequal cohort triggers a `FairnessBreach`; equalised cohort passes; verifies the reports endpoint reflects the breach |
| `attestation_replay_test` | Fresh nonce accepted; replayed nonce rejected (401); stale nonce (>60s) rejected regardless |
| `vendor_failover_test` | Primary vendor unhealthy -> 1:N still completes via secondary (EC-CON-05 flapping-link scenario) |

Everything needing the deployed stack (PostgreSQL + Kafka + running
abis-api) is gated behind the `cross-crate-integration` feature: ignored by
default, never fake-passed; run with
`cargo test -p abis-integration-tests --features cross-crate-integration -- --include-ignored`
after `just up` (`Cargo.toml` comments). A sixth file,
`tests/integration_test.rs`, remains a placeholder and is not wired.

---

# Operations and governance

All crates here are library-only today (no in-workspace consumers) unless
noted. Test counts are external `tests/*.rs` files.

- **abis-accelerator.** GPU/FPGA offload interfaces with CPU fallback (`src/lib.rs:1-4`). A
  dispatcher picks an accelerator from gallery size, latency, and energy budgets:
  `select_accelerator` (`src/dispatcher.rs:16`). Traits `GpuMatcher` (`src/gpu.rs:6`) and
  `FpgaMatcher` (`src/fpga.rs:6`) have stub implementations (`StubGpuMatcher` `gpu.rs:12`,
  `StubFpgaMatcher` `fpga.rs:13`); the working path is `CpuFallbackMatcher`
  (`src/cpu_fallback.rs:6`, `batch_1n` `:9`). Verification: 12 tests.

- **abis-bulk-ops.** Batch biometric operations with idempotent resumption (`src/lib.rs:1-4`);
  seven op types: enrol, dedup, delete, export, rewrap, import, retire. Per-op processors
  (`process_enrol_batch` `src/enrol.rs:15`, `find_duplicates`/`process_dedup_batch`
  `src/dedup.rs:14,27`, `export_to_csv` `src/export.rs:7`, watchlist CSV import
  `src/import.rs:16`, retire `src/retire.rs:15`, rewrap `src/rewrap.rs:6`), lifecycle dispatcher
  (`validate_runnable` / `pause` / `cancel`, `src/dispatcher.rs:9-35`), and checkpointed progress
  with CSV failure manifests (`record_progress` `src/progress.rs:9`, `write_manifest` `:38`).
  Verification: 21 tests.

- **abis-drift-monitor.** Feature-space drift detection against baseline snapshots over rolling
  24h/7d/30d windows (`src/lib.rs:1-3`). Statistics: PSI (`src/detector.rs:9`), KS statistic
  (`:32`), Jensen-Shannon divergence (`:54`), Wasserstein distance (`:83`);
  `DriftDetector::detect` (`:129`); adaptive threshold suggestion (`src/adaptive.rs:7`); window
  aggregation (`src/window.rs:7,11`); alert hook (`maybe_alert`, `src/alert.rs:10`). Verification:
  13 tests.

- **abis-threshold-tuning.** Per-cohort ROC-based threshold selection targeting a chosen FMR, with
  n=1000 bootstrap 95% CIs (`src/lib.rs:1-4`). `compute_roc` (`src/tuner.rs:17`),
  `select_threshold` (`:49`), `ThresholdTuner::tune` (`:71,87`); activation, persistence
  (`repo.rs`), and an axum `api.rs`. Verification: 15 tests.

- **abis-explainability.** Explainability toolkit: gradient saliency maps (`compute_saliency`,
  `src/saliency.rs:23`), local SHAP (`LocalShapExplainer`, `src/shap.rs:9`), LIME
  (`LocalLimeExplainer`, `src/lime.rs:13`), attention maps with a uniform placeholder emitter
  (`UniformAttentionEmitter`, `src/attention.rs:21`), decision reasoning text
  (`generate_reasoning_text`, `src/reasoning.rs:14`), and SVG minutiae overlays (`generate_svg`,
  `src/overlay.rs:27`). Like fusion-advanced, it has no crate-level doc header. Verification: 15
  tests.

- **abis-cluster-analysis.** Nightly multi-way duplicate detection via connected components over
  adjudication-confirmed pairs from the last 30 days (`src/lib.rs:1-4`): `connected_components`
  (`src/cluster.rs:58`), `detect_clusters` (`src/detector.rs:10`), merge/split/false-alarm
  resolutions (`src/resolver.rs:8,17,26`). Verification: 15 tests.

- **abis-twin-registry.** Twin group management because identical twins land in the ambiguous zone
  of 1:N matching (`src/lib.rs:1-4`). Candidate detection (`detect_twin_candidates`,
  `src/detector.rs:16`; face-iris correlation check `:66`), group lifecycle (`create_twin_group`
  `src/registry.rs:11`, `add_member` `:28`), civil-registry cross-reference
  (`verify_from_civil_registry` `:42`). Verification: 13 tests.

- **abis-watchlist.** Multi-modal watchlist screening: `WatchlistManager` (`src/manager.rs:22`,
  add/remove/sunset/hit-recording `:37-175`), per-list Bloom filter + `WatchlistIndex`
  (`src/index.rs:4,68`) combined in `MultiIndex`, and entry points `check_watchlist`
  (`src/matcher.rs:8`) plus Bloom-accelerated `check_watchlist_fast` (`:46`). Verification: 17
  tests.

- **abis-search-planner.** Cost-based 1:N query planner: plans the cheapest sequence of
  demographic_prune -> face_ANN -> fingerprint_exact -> iris_exact and executes concurrently
  across shards with early termination (`src/lib.rs:1-4`). `QueryPlanner::plan`
  (`src/plan.rs:16,29`), `CostModel` (`src/estimator.rs:5`, `estimate_step_cost` `:24`),
  concurrent execution (`execute_plan`, `src/executor.rs:18`). Verification: 15 tests.

- **abis-perf-monitor.** Continuous FMR/FNMR and rank-N accuracy monitoring on a labelled holdout
  set every 6 hours, with daily/weekly/monthly rollups (`src/lib.rs:1-4`): `compute_fmr`
  (`src/pmr.rs:18`), `compute_fnmr` (`:30`), `compute_rank_n_accuracy` (`:42`), `run_pmr` (`:57`).
  Verification: 13 tests.

- **abis-reports.** Scheduled reporting engine. Renderers behind a `Renderer` trait
  (`src/render/mod.rs:69`) for csv, html, json, parquet, pdf, png, xlsx (`src/render/*.rs`); cron
  scheduling (`next_fire`, `src/scheduler.rs:81`); signed envelopes (`SignEnvelope`,
  `src/signer.rs:11`, `verify` `:98`); extraction and pipeline glue in `extractor.rs` /
  `anchor.rs` / `pipeline.rs`. Verification: 27 tests.

- **abis-reports-adhoc.** Ad-hoc query reporting with safety linting: SQL builder with allow-
  listed tables and joins (`QueryBuilder::compile`, `src/query_builder.rs:52`), SQL validation
  producing a `LintReport` with severity diagnostics (`validate_sql`, `src/validator.rs:87`;
  `is_safe` `:57`), preview, publish (`PublishResult`, `src/publish.rs:8`), scheduled ad-hoc runs,
  and event emission. Renders through the two render crates below. Verification: 27 tests.

- **abis-report-publisher.** Report publication and delivery (RPT-B, `src/lib.rs:1-4`) via seven
  channels under `src/channels/`: email, webhook, kafka, sftp, gov_printer, kbc_sftp, api_pull.
  Delivery orchestration in `src/pipeline.rs`, subscription repository, metrics snapshot in
  `src/metrics.rs`, axum endpoints in `src/api.rs`. Verification: 15 tests.

- **abis-render-docx / abis-render-markdown.** Document renderers consumed by `abis-reports-
  adhoc`. DOCX: `render_docx` (`abis-render-docx/src/render.rs:127`) driven by `DocxTemplate`
  (`:89`), `Section` variants (`:59`), page size/orientation (`:39,27`), callout styles (`:51`).
  Markdown: `render_markdown` (`abis-render-markdown/src/render.rs:97`) with `MdSection` (`:32`),
  `TableAlignment` (`:11`), callout styles, options/output types (`:84,91`). Verification: 13 and
  11 tests respectively.

---

## Closing notes (honest assessment)

- **Unwired middleware.** `spire_auth` and `require_idempotency_key` are
  implemented but never layered onto the router; only the
  placeholder-signature `cosign_response` middleware runs. Idempotency
  covers the match endpoints via explicit `lookup`/`record` calls.
- **Placeholder response signing.** `x-eel-signature: stub-cosign-v1` is a
  constant, not a signature (`abis-api/src/middleware.rs:170`).
- **Static handlers.** Enrolment, kit, admin, and reports endpoints largely
  return canned responses; `match_11` uses `MockMatcher`. Real persistence
  is limited to the idempotency table and match request/result rows.
- **Stub backends.** `SoftwarePad` always returns Live; GPU/FPGA matchers,
  `GrpcVendor`, fusion-advanced's `pca_reduce`, and the explainability
  attention emitter are documented stubs or placeholders.
- **Orphans.** All operations/governance crates above, plus
  `abis-enrolment` and `abis-fp-vendor-peak`, have no in-workspace
  consumers yet.
- **Legacy stub retained deliberately.** `abis-matcher/src/adjudication.rs`
  remains while `abis-adjudication` is the real implementation.
- **Thin tests in places.** `abis-vendor` ships one external test for a
  six-module surface; `abis-api` has five small inline tests and relies on
  the feature-gated, default-ignored integration harness for HTTP coverage.
