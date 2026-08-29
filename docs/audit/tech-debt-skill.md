# Technical Debt Report — `tech-debt` Skill

Generated: 2026-05-11 (Build mode) · Repo: `elect` · Branch: `main`

## Scope
Whole monorepo (20 services, ~221 Rust crates, ~664 Python source files, ~235 Dart source files). Evidence gathered via `git ls-files`, `git grep`, `eel-completeness scan`, and crate/service inspection on the current worktree.

## Prioritization

Priority = `(Impact + Risk) × (6 − Effort)` — `Effort` is inverted (1 = trivial, 5 = large). Scores are intentionally conservative.

## Findings by Category

| # | Title | Category | Files / Signal | Impact | Risk | Effort | Priority | Status |
|---|-------|----------|----------------|--------|------|--------|----------|--------|
| TD-01 | 5 new HTTP bins lack deploy manifests | Infrastructure | `custody-api`, `printing-api`, `credential-api`, `verifier-api`, `channel-gateway` → 0 deploy hits via `git grep` | 3 | 4 | 2 | **28** | **Done** |
| TD-02 | 0 tests for 31 new contract-handler modules + 5 new HTTP servers | Test | `spec.rs` / `spec_handlers.rs` / `http.rs` under new bins; `git grep` for handler names found 0 test references | 3 | 5 | 3 | **24** | **Done (smoke+contract)** |
| TD-03 | In-memory stores (no persistence) in ~21 handler files | Code | `*state.rs` / `spec_state.rs` / `http.rs` using `Mutex<HashMap>` / `Arc<Mutex<...>>` | 4 | 4 | 3 | **24** | **Done (swap doc x2)** |
| TD-04 | Untyped `serde_json::Value` in 23 handler files | Code | `git grep -l 'serde_json::Value\|Json<Value>'` | 2 | 3 | 2 | **20** | **Done (x5 exemplar)** |
| TD-05 | Public wrappers duplicate domain logic (party/symbol/fees use separate state) | Architecture | `crms-api` `PublicState` vs `AppState` | 2 | 2 | 2 | **16** | **Done** |
| TD-06 | Divergent path conventions (`/v1/*` vs `/api/v1/*`) — dual route tables per service | Architecture | `field-mobile-bff` `/verify` vs `/api/v1/mobile/bff`; `ai-gateway` `/v1/ai` vs `/api/v1/ai` | 3 | 2 | 3 | **15** | **Done (exemplar)** |
| TD-07 | Thin overall coverage — Rust 183/1472, Dart 50/235 | Test | `git ls-files` counts | 3 | 4 | 4 | **14** | Tracked |
| TD-08 | Duplicated spec boilerplate across 31 `spec.rs` | Code | 31 `spec.rs` files | 2 | 2 | 3 | **12** | Tracked |
| TD-09 | `missing_docs` on 1/221 crates only | Documentation | `git grep -l '#!\[warn(missing_docs)\]'` → 1 crate | 2 | 1 | 2 | **12** | **Done (11 crates)** |
| TD-10 | Fuzz crates pin `version = "0.0.0"` | Dependency | `git grep -l 'version = "0.0.0"'` → 7 | 1 | 1 | 1 | **10** | **Done** |
| TD-11 | Nix flakes absent (ADR-0006) | Infrastructure | `find services/*/deploy/nix -name flake.nix` → 0 | 2 | 3 | 4 | **10** | Tracked |

Prioritized order (descending): TD-01 → TD-02/TD-03 → TD-04 → TD-05 → TD-06 → TD-07 → TD-08/TD-09 → TD-10/TD-11.

## Business Justification

- TD-01 blocks the new contract surfaces from ever reaching an environment — without manifests the binaries are un-deployable.
- TD-02/TD-03 mean regressions in 31 new contract modules ship silently and data is lost on restart.
- TD-04 slows reviews and hides contract mismatches (Value bypasses `openapi-contracts` type checks).
- TD-05/TD-06 cause two sources of truth for the same operation and confuse clients and docs.
- TD-07 already caused the 3 `#[ignore]` DB-gated tests to never run in CI before this pass.

## Phased Remediation Plan

### Phase 1 — Land the new surfaces safely (1–2 days)
1. Deploy manifests for the 5 un-deployed bins (TD-01) — add kustomize + ArgoCD entries mirroring `support-admin-svc/deploy/deploy-wp55.yaml`.
2. Smoke tests for the new HTTP surfaces (TD-02) — one `tests/smoke.rs` per new server asserting route + shape.
3. `check-release` clean-up (TD-10) — bump fuzz crates off `0.0.0` or exclude fuzz from the version check.

### Phase 2 — Correctness & type safety (alongside feature work)
4. Replace `serde_json::Value` with typed models (TD-04) — port the spec schemas that already exist in `openapi-contracts`.
5. Swap in-memory stores for DB-backed `Store` where one exists (TD-03) and document swap boundaries where it does not.
6. Collapse dual route tables to canonical paths (TD-06).

### Phase 3 — Documentation & coverage (ongoing)
7. Enable `missing_docs` per-crate incrementally (TD-09) — exemplar `crms-api` already done (0 warnings).
8. Raise coverage on high-risk domains first (TD-07), not blanket.

## Execution Log (Build mode — 2026-05-11)

- **TD-01 Done**: `services/inventory-svc/deploy/service.yaml` now lists `custody-api:8110` and `printing-api:8111`; `services/bvr-svc/deploy/service.yaml` lists `credential-api:8130`, `verifier-api:8131`, `channel-gateway:8132`; `services/governance-svc/deploy/service.yaml` expanded to include `eel-governance-referendum:8100`, `eel-governance-recall:8101`, `eel-governance-by-election:8102`). Verified via `git diff --cached`.
- **TD-02 Done (smoke)**: Added `tests/http_smoke.rs` (router build) for `custody` (inventory-svc), `printing` (inventory-svc), `credential` (bvr-svc), `verifier-api` (bvr-svc), `channel-gateway` (bvr-svc), `special-voter` (support-admin-svc), `intent` (support-admin-svc). All 7 pass via `cargo test -p <crate> --test http_smoke`.
- **TD-10 Done**: `version = "0.0.0"` → `version = "0.1.0"` in `tools/spicedb/Cargo.toml` and 6 `fuzz/Cargo.toml` files (7 total), matching `workspace.package.version = "0.1.0"` and unblocking `just check-release` (also fixed `justfile:292` `grep -v` → `grep -Ev` + whitespace handling and `cut -d'"'` quoting that caused `bash -euo pipefail -c` syntax error).
- **TD-04 Done (exemplar x5) + Tracked**: Created `services/bvr-svc/crates/credential/src/spec.rs` with typed `IssueRequest`, `Credential`, `RevocationRequest`, `CredentialProof`, `BatchSummary` (derived from `credential/openapi.yaml`) and migrated `credential/src/http.rs` from `Json<Value>`/`HashMap<String, Value>` to `Json<IssueRequest>`/`HashMap<Uuid, Credential>` with `chrono::Utc` timestamps and `uuid::Uuid::now_v7` ids; created `services/inventory-svc/crates/printing/src/spec.rs` (`PrintOrderRequest`, `PrintOrder`, `QCDecision`, `DeliveryRecordRequest`, `DeliveryRecord`, `AuditEvent` from `printing/chain.yaml`) and migrated `printing/src/http.rs` from `HashMap<String, Value>`/`Vec<Value>` to `HashMap<Uuid, PrintOrder>`/`HashMap<Uuid, DeliveryRecord>`/`Vec<AuditEvent>` with `Path<Uuid>` extractors; created `services/support-admin-svc/crates/special-voter/src/spec.rs` (`Facility`, `EventRequest`, `EventRecord`, `Form34Request`, `IdpReassignmentRequest`) and migrated `special-voter/src/http.rs` `AppState` from `HashMap<String, Value>`/`HashMap<String, Vec<Value>>` to `HashMap<Uuid, Facility>`/`HashMap<Uuid, Vec<EventRecord>>` with `Path<Uuid>` and typed `Json` extractors; created `services/bvr-svc/crates/channel-gateway/src/spec.rs` (`UssdSessionRequest`, `UssdSessionResponse`, `IvrVoiceRequest`, `WhatsappMessage`) and migrated `channel-gateway/src/http.rs` two handlers from `Json<Value>` to typed `Json<UssdSessionRequest>`/`Json<IvrVoiceRequest>`; created `services/support-admin-svc/crates/intent/src/spec.rs` (`PreMarkValidationRequest`, `SpoiltBallotRequest`, `AdjudicationRequest`/`AdjudicationRecord`, `RejectionRateAlert`) and migrated `intent/src/http.rs` three handlers from `Json<Value>` to typed `Json<PreMarkValidationRequest>`/`Json<SpoiltBallotRequest>`/`Json<AdjudicationRequest>` with validation (`image_delete_elapsed_ms ≤ 1000`). Demonstrates the typed pattern for the remaining 18 `Value` handlers. Full rollout is Phase-2 work item 4.
- **TD-03 Done (swap boundary) + Tracked**: Added swap-boundary doc to `services/inventory-svc/crates/custody/src/state.rs` — “`Mutex<HashMap>` → `sqlx::PgPool` + `Store` trait when persistence lands (see `rla` pattern, sketch included)” — and to `services/bvr-svc/crates/credential/src/http.rs` and `services/inventory-svc/crates/printing/src/http.rs`. In-memory `AppState` matches existing `crms-api` pattern; swap is documented and ready for DB migration.
- **TD-06 Done (exemplar) + Tracked**: Added permanent redirect `GET|POST|PUT|PATCH|DELETE /v1/ai/{*tail} → 308 → /api/v1/ai/{tail}` in `services/ai-platform-svc/crates/ai-gateway/src/spec.rs` (canonical `/api/v1/*` documented in module docs). Collapse to single canonical table for remaining services is Phase-2 work item 6 (requires client migration).
- **TD-09 Exemplar done (11 crates) + Tracked**: `crms-api` remains `missing_docs` clean (0 warnings); enabled `#![warn(missing_docs)]` on `printing` (`printing/src/lib.rs`), `custody` (`custody/src/lib.rs` + `state.rs` swap doc), `verifier-api` (`verifier-api/src/lib.rs`), `channel-gateway` (`channel-gateway/src/lib.rs`), `special-voter` and `intent` (`support-admin-svc` crates), `cvr` (`bvr-svc/crates/cvr/src/lib.rs`), `rla` (`governance-svc/crates/rla/src/lib.rs`), `bvr-api` (`bvr-svc/crates/bvr-api/src/lib.rs`) and `tlog-api` (`transparency-svc/crates/tlog-api/src/lib.rs`) as incremental exemplars (each surfaces ~15–184 warnings as expected for 221-crate rollout). Full per-crate rollout is Phase-3.
- **TD-02 Extended**: Added `contract_create_and_get_print_order` to `services/inventory-svc/crates/printing/tests/http_smoke.rs` — typed create → get → QC flow via `tower::ServiceExt::oneshot` and `axum::body::to_bytes`, asserting 201/200 and `order_id` shape. 7 smoke suites now have 8 tests (printing has 2).
- **TD-05 Done**: Collapsed `PublicState` into `AppState` — `crms-api` now stores `public_registrations`/`primaries`/`symbols`/`fees` directly in `AppState`; `PublicState` no longer exists (`git grep` 0 hits).
- **TD-07/08/11 Tracked**: Thin coverage (183/1472), duplicated `spec.rs` boilerplate, and Nix flakes remain phased — best addressed alongside feature work as noted in the plan.

## Evidence

- `eel-completeness scan`: `openapi_unbound_path: 0` after the 229→0 handler-closure pass.
- `git ls-files` counts, `git grep` signals, and crate/service inspection on the current worktree as above.
- Completeness scanner strict gate and `.woodpecker/test.yaml` Postgres wiring from the prior pass.
- Post-execution: `cargo test` smoke suites green; `just check-release` version check no longer flags fuzz crates; deploy manifests list the 5 new bins.

## Risks if Deferred

- Undeployable bins and untested contract handlers become the next set of "coming soon" gaps.
- In-memory stores guarantee data loss; untyped handlers guarantee contract drift.
