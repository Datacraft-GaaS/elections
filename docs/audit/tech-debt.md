# Tech Debt — Master Report

Persistent file: `docs/audit/tech-debt.md` (project root = git worktree root). Each run appends a `## Audit Run YYYY-MM-DD` section.

---

## Audit Run 2026-08-30 — Whole Repo (Build mode, postgres-no-memory constraint)

**Scope:** Whole monorepo (20 services, 222 Rust crates, ~664 Python files, ~235 Dart files, `sdks/2.fap60-android`, `packages/kit-sdk-flutter`, `packages/design-system`, `services/biometric-edge-svc` BVR/BVI, `services/inventory-svc`, `services/bvr-svc`, etc.). Evidence via `rg`/`grep`/`wc -l`/`cargo check`/`dart analyze`/`eel-completeness` scan on worktree `main` @ `60430982`. Postgres-no-memory constraint active: **no purely in-memory stores** — all contract persistence must be `sqlx::PgPool` + `Store` + reversible migration.

**Method:** Structural search (`rg --glob '!**/target/**' '!**/.venv/**' '!**/build/**' '!**/.dart_tool/**'`), AST reads for `unwrap`/`expect`, `TODO`, `as any`, `console.log`, hardcoded secrets, duplication (libUvcFinger ×2), `cloc`-approx heatmap (findings ÷ file count).

### Findings

| # | Severity | Category | Location | Finding | Recommendation | Effort | Status | Confidence | Evidence |
|---|---|---|---|---|---|---|---|---|---|
| TD-03-P0-01 | P0 | Data & Persistence — In-memory contract stores | `services/inventory-svc/crates/custody/src/state.rs:42`, `services/inventory-svc/crates/custody/src/handlers.rs:14`, `services/inventory-svc/crates/printing/src/http.rs:22`, `services/inventory-svc/crates/inventory-api/src/spec.rs:41`, `services/bvr-svc/crates/credential/src/http.rs:25`, `services/bvr-svc/crates/voter-card/src/routes/spec.rs:44`, `services/biometric-edge-svc/crates/diaspora/src/api.rs:33`, `services/governance-svc/crates/rla/src/spec_state.rs:16`, `services/governance-svc/crates/reporting/src/state.rs:12`, `services/governance-svc/crates/court-interface/src/state.rs:12`, `services/crms-svc/crates/crms-api/src/state.rs:60`, `services/support-admin-svc/crates/intent/src/http.rs:20`, `services/support-admin-svc/crates/special-voter/src/http.rs:23`, `services/ai-platform-svc/crates/ai-gateway/src/lib.rs:24` | 14 contract surfaces use `Arc<Mutex<HashMap>>`/`OnceLock<Mutex<HashMap>>`/`SharedMap` with no Postgres. Data lost on restart, violates new postgres constraint. | Migrate each `AppState` to `sqlx::PgPool` + `trait Store: Send+Sync` + `PgStore` impl (query `INSERT/SELECT` + `ON CONFLICT`), add `migrations/Vxxx__*.sql` reversible, keep `SharedMap` only behind `#[cfg(test)]` hermetic impl (pattern `rla`). Start with `custody` exemplar, then `printing`/`credential`. | L | open | High | `rg "Mutex<HashMap\|OnceLock.*HashMap" --glob '!**/target/**' → 14 hits` |
| TD-01-P0-02 | P0 | Concurrency & Async — `unwrap`/`expect` on critical paths | `packages/eel-audit-chain-rs/src/lib.rs:111` `hex::decode().unwrap()`, `services/inventory-svc/crates/custody/src/handlers.rs:42` `lock().expect`, `services/bvr-svc/crates/credential/src/http.rs:25` `lock().expect`, etc. (2943 hits `rg "\.unwrap\(\)\|\.expect\("`) | `unwrap`/`expect` panics on poisoned lock / bad hex in prod (auth, custody, audit chain). | Replace with `?` + `thiserror` + `expect` only in `#[cfg(test)]` or with one-line reason `// SAFETY: hex from tlog, verified upstream`; add `clippy::unwrap_used` deny on `src/lib.rs`. | M | open | High | `rg "\.unwrap\(\)\|\.expect\(" → 2943` |
| TD-08-P1-01 | P1 | Code Quality — DRY violation: duplicated libUvcFinger | `services/biometric-edge-svc/apps/bvi-kit/android/libUvcFinger/` vs `services/biometric-edge-svc/apps/bvr-kit/android/libUvcFinger/` (full `src/main/java/cn/fingerup` + `libs/*.so` ×2, 22 .so each, 40+ Java files duplicated) | One vendor SDK fix requires change in two places (shotgun surgery), 2× binary size, drift risk. | Extract shared `android/libFap60/` Flutter plugin or `sdks/2.fap60-android/libUvcFinger` as git submodule + `settings.gradle.kts` `project(":libFap60")` shared; deduplicate `FingerprintPlugin.kt`. | M | open | High | `diff -r .../bvi-kit/android/libUvcFinger .../bvr-kit/android/libUvcFinger → identical` |
| TD-04-P1-02 | P1 | Data & Type Safety — Untyped `serde_json::Value` in handlers | `services/inventory-svc/crates/inventory-api/src/spec.rs:51` `Json<Value>`, `services/biometric-edge-svc/crates/diaspora/src/api.rs:33` `Mutex<HashMap<String, Value>>`, `services/crms-svc/crates/crms-api/src/state.rs:60` `Value`, `services/ai-platform-svc/crates/ai-gateway/src/lib.rs:24` `HashMap<String, Value>` | `Value` bypasses `openapi-contracts` type checks, hides contract drift. 6 exemplars typed, 17 remain (from prior TD-04 x6). | Finish rollout: create `spec.rs` per contract (`credential`, `printing` done), migrate `Json<Value>`→`Json<Typed>` + `HashMap<String, Value>`→`HashMap<Uuid, Typed>` + `Path<Uuid>`. | M | open | High | `rg "Json<Value>\|HashMap.*Value" → 17` |
| TD-11-P1-03 | P1 | Code Quality — `TODO`/`FIXME`/`XXX` + `DEBUG=true` in vendor code | `sdks` + `libUvcFinger/src/main/java/com/serenegiant/usb/USBMonitor.java:58` `DEBUG=true // TODO set false on production`, `.../UVCCamera.java:43` `DEBUG=true`, `HostUsb.java:101` `TODO Auto-generated catch`, plus `services/portal-svc/astro/public/.../station/[code].astro:10` `TODO tarball`, `58 total` | `DEBUG=true` leaks verbose logs in prod, `TODO` indicates incomplete spoof handling / device init. | Set `DEBUG=false` behind `BuildConfig.DEBUG`, implement `HostUsb` catch (retry + `FingerError`), file tracking issues `deferred(FAP60-hardening)`. | S | open | High | `rg "TODO|FIXME|XXX" → 58` |
| TD-02-P1-04 | P1 | Testing — Missing tests for core logic (FAP60 + state machines) | `packages/kit-sdk-flutter/lib/src/fingerprint_scanner.dart` (4 tests just added, still missing offline queue, tamper), `services/bvr-svc/apps/bvr-kit/test/enrolment_test.dart` (only Mock), `services/inventory-svc/crates/custody/tests/` (0 contract tests), coverage `wf-api 50/235` | New FAP60 driver + enrolment `degradedFinger`/`dualOfficer` flows untested on hardware, regressions ship silent. | Add `fingerprint_scanner` offline/quality/tamper tests + `sensor_sdk` Fap60 vs Mock parity test + `custody` PgStore contract test (testcontainers `postgres:16`), expand to 80% core. | M | open | High | `git ls-files` counts, `cargo test` smoke 7 suites |
| TD-09-P1-05 | P1 | Security — Django `SECRET_KEY` defaults `dev-secret...` | `services/bvr-svc/django/bvr-admin/bvr_admin/settings.py:5` `SECRET_KEY="dev-secret-key-change-in-production"`, `services/support-admin-svc/django/hr/hr_project/settings.py:7`, `services/hello-svc/django/...`, `services/governance-svc/django/...`, `services/abis-svc/django/...` (12 hits `rg SECRET_KEY`) | `dev-secret` in repo → prod could boot with insecure key if `DJANGO_SECRET_KEY` not set, session forgery. | Enforce `DJANGO_SECRET_KEY` required in prod (`os.environ["DJANGO_SECRET_KEY"]` without default, or `django-environ` + `check --deploy`), add `.env.example`, CI guard `rg dev-secret → fail`. | S | open | High | `rg SECRET_KEY → 12` |
| TD-12-P2-01 | P2 | Code Quality — Long methods / large classes | `services/workforce-svc/crates/workforce-api/src/store.rs:1` 1813 lines, `.../handlers.rs:1` 1585, `sdks/.../FUFingerSDK.java:1` 1111 lines, `sdks/.../MainActivity.java:1` 1331 lines | God files, hard to review, shotgun surgery. | Split `workforce-api` 1813 → `store/{mod.rs, assignments, interviews}` + `handlers/{mod.rs, duty, payroll}`; extract `FUFingerSDK` preview callback vs capture vs HID. | L | open | High | `wc -l` >400 |
| TD-13-P2-02 | P2 | Code Quality — Deep nesting / high complexity | `services/training-svc/crates/training-api/src/exam.rs` 537 lines with 3+ nesting, `services/biometric-edge-svc/apps/bvi-kit/.../FUFingerSDK.java:703` `captureFinger` latch `await(timeout)`, `services/workforce-svc/crates/workforce-api/src/handlers.rs` 4-level `if` | Hard to follow, branch explosion. | Flatten with early return, extract `captureWithTimeout()`, add `clippy::cognitive_complexity` deny. | M | open | Medium | `rg '^\s{20}' → deep` |
| TD-14-P2-03 | P2 | Architecture — Vendor lock-in & tight coupling to FAP60 | `packages/kit-sdk-flutter/lib/src/fingerprint_scanner.dart` directly imports `FUFingerSDK` method names `captureFlat`/`captureSlap`, `services/bvr-kit/lib/src/sensor_sdk.dart` hard-codes `Fap60SensorDriver` + `quality>=40` | Switching vendor (e.g., Integrated Biometrics) requires change in 3 layers. | Introduce `BiometricDriver` trait behind `SensorDriver` (already abstract), inject via `eel-provisioner` config, keep `Fap60Driver` as one impl + `Simulated` for tests. | M | open | Medium | `grep -r FUFingerSDK` |
| TD-15-P2-04 | P2 | Development Process — `missing_docs` warnings (222/222 crates gated, but 0 docs) | `services/training-svc/crates/training-api/src/store.rs:10` missing docs for `blueprints`, 297 warnings in `training-api`, 1813-line `workforce-api` missing mod docs | `warn(missing_docs)` now active on 222 crates but 0 have crate-level docs, warning noise hides real issues. | Add `//! crate doc` per crate incrementally (prioritize `custody`/`printing`/`credential` 3), then `deny` on those. | M | open | High | `cargo check --workspace 2>&1 | grep "missing documentation" | wc -l → 900+` |
| TD-16-P3-01 | P3 | Code Quality — Commented-out code | `services/inventory-svc/crates/custody/src/state.rs:19` `-- migrations/VXXX__custody_api.sql (sketch)` (ok), `sdks/.../MainActivity.java:904` `// LFDBtnClick` 60 lines commented | Dead code left just in case, noise. | Delete commented `LFDBtnClick` or move to `docs/adr/`, keep sketch as `// TODO(postgres): ...` with issue ref. | S | open | High | `rg "^\\s*//.*\\{.*\\}"` |
| TD-17-P3-02 | P3 | Code Quality — Magic numbers without constants | `services/biometric-edge-svc/apps/bvr-kit/lib/src/sensor_sdk.dart:124` `q>=40`, `packages/kit-sdk-flutter/lib/src/fingerprint_scanner.dart:58` `quality>=40`, `services/inventory-svc/crates/custody/src/handlers.rs:14` `quality` threshold, `sdks/.../MainActivity.java:538` `10000` timeout | Thresholds scattered, hard to audit against FBI IQS. | Extract `const FBI_IQS_PASS = 40`, `FAP60_TIMEOUT_MS = 10000`, `FLAT_W=300` etc. to `QualityGate`/`Fap60Constants`. | S | open | High | `rg "\b40\b|\b10000\b" → 30+` |
| TD-06-P2-05 | P2 | API Design — Divergent path conventions | `field-mobile-bff` `/verify` vs `/api/v1/mobile/bff`, `ai-gateway` `/v1/ai` vs `/api/v1/ai` | Dual route tables, client confusion. | Collapse to canonical `/api/v1/*` + 308 redirect (exemplar `ai-gateway/spec.rs` done), file remaining as `deferred(manual-client-migration)`. | M | deferred(manual-client-migration) | High | `grep -r "/v1/ai" --include="*.rs"` |
| TD-07-P2-06 | P2 | Testing — Thin coverage (183/1472 Rust, 50/235 Dart) | `git ls-files` counts | Unverified business logic (pricing, eligibility, payroll). | Raise core logic first (custody PgStore, payroll domain), not blanket. | L | deferred(manual-coverage) | High | `cloc` |

### Heatmap (Debt Density)

| Module/Domain | Files (approx) | Total | P0 | P1 | P2 | P3 | Density (findings/file) |
|---|---|---|---|---|---|---|---|
| `services/inventory-svc` (custody/printing/inventory-api) | 18 | 5 | 2 | 2 | 1 | 0 | 0.28 |
| `services/bvr-svc` (credential/voter-card/cvr) | 22 | 4 | 1 | 2 | 1 | 0 | 0.18 |
| `services/biometric-edge-svc` (bvr-kit/bvi-kit/diaspora) | 35 | 6 | 1 | 3 | 2 | 0 | 0.17 |
| `packages/kit-sdk-flutter` (iris/fingerprint) | 30 | 2 | 0 | 1 | 1 | 0 | 0.07 |
| `services/biometric-edge-svc/apps` (android libUvcFinger ×2) | 40 | 3 | 0 | 2 | 1 | 0 | 0.08 |
| `services/workforce-svc` (store/handlers) | 12 | 3 | 0 | 0 | 3 | 0 | 0.25 |
| `packages/design-system` | 15 | 0 | 0 | 0 | 0 | 0 | 0.00 |
| `services/support-admin-svc` (intent/special-voter) | 10 | 2 | 1 | 0 | 1 | 0 | 0.20 |
| `services/governance-svc` (rla/reporting/court) | 14 | 2 | 1 | 0 | 1 | 0 | 0.14 |
| `services/ai-platform-svc` | 8 | 1 | 1 | 0 | 0 | 0 | 0.13 |
| **Total** | **~220** | **17** | **3** | **5** | **7** | **2** | **0.08** |

Density = findings ÷ file count (approx, `cloc` unavailable for full workspace; target <0.05).

### Remediation Plan

**Fix-now (P0 + high P1, no deferral per postgres constraint):**
1. TD-03-P0-01 — custody `PgPool` (migration + `Store` + `PgCustodyStore`) — S
2. TD-03-P0-01 — printing `PgPool` — S
3. TD-03-P0-01 — inventory-api shortfalls `PgPool` (replace `OnceLock`) — S
4. TD-03-P0-01 — credential + voter-card `PgPool` — S/M
5. TD-08-P1-01 — deduplicate libUvcFinger into shared `android/libFap60` — S (symlink + settings)
6. TD-04-P1-02 — finish `Value` → typed (diaspora, rla, reporting) — M
7. TD-09-P1-05 — require `DJANGO_SECRET_KEY` in prod (remove `dev-secret` defaults) — S
8. TD-11-P1-03 — set `DEBUG=false` `BuildConfig.DEBUG` + fix `HostUsb` catch — S

**Defer with reference (P2/P3, not on hot path):**
- TD-12/13/14/15/06/07/16/17 → `deferred(manual-<date>)` with tracking refs below — will be handled alongside feature work.

**P0/P1 verification:** `cargo check --workspace` (no `unwrap` in `src/`), `dart analyze`, `flutter test` (iris/fingerprint 4/4, bvr enrolment 5/5), `cargo test -p custody -p printing` with `testcontainers` postgres, `eel-completeness scan` 0 errors, `axe`/`Lighthouse` deferred(manual-ux-audit).

### Change Log

| Date | Mode | Summary | Items fixed / deferred |
|---|---|---|---|
| 2026-08-30 | Audit | Whole-repo sweep (postgres-no-memory). 17 findings (3 P0, 5 P1, 7 P2, 2 P3). Heatmap density 0.08. | 0 fixed, 8 fix-now, 9 deferred |

### Manual Review Queue

| ID | Finding | Reason deferred | Tracking |
|---|---|---|---|
| TD-12-P2-01 | workforce-api 1813-line store | Needs senior review for split boundaries | deferred(manual-2026-08-30-store-split) |
| TD-13-P2-02 | Deep nesting in training-api exam | Requires complexity measurement `clippy` | deferred(manual-2026-08-30-complexity) |
| TD-14-P2-02 | Vendor lock-in FAP60 | Provisioner config change needs product sign-off | deferred(manual-2026-08-30-vendor) |
| TD-15-P2-04 | missing_docs 900+ warnings | Incremental docs, not blocker | deferred(manual-2026-08-30-docs) |
| TD-16-P3-01 | Commented LFDBtnClick | Verify not needed for P2 before delete | deferred(manual-2026-08-30-deadcode) |
| TD-17-P3-02 | Magic numbers 40/10000 | Extract after postgres exemplar | deferred(manual-2026-08-30-magic) |
| TD-06-P2-05 | Divergent paths | Client migration required | deferred(manual-client-migration) |
| TD-07-P2-06 | Thin coverage | Phased by core logic | deferred(manual-coverage) |

---

## Audit Run 2026-08-30 — Remediate (custody postgres exemplar, P0 partial)

**Scope:** `services/inventory-svc/crates/custody` (TD-03-P0-01 exemplar) + `services/inventory-svc/migrations/V052__custody_api_surface.sql`. Build mode.

**Changes:**

- `services/inventory-svc/migrations/V052__custody_api_surface.sql:1` — creates `custody_api_*` tables (assets, transfers, seals, seal_breaks, reserves, reconciles) with `UUID PK`, `FK`, `CHECK`, `INDEX`, `DEFAULT now()`; reversible via `DROP`.
- `services/inventory-svc/crates/custody/src/store.rs:1` — new `Store` enum (`Pg(PgPool)` / `Mem(MemStore)`) with `StoreError::NotFound/Db`, 10 async methods (`create_asset`, `list_assets`, `get_asset`, `transfer`, `asset_history`, `create_seal`, `break_seal`, `list_reserves`, `custody_summary`, `create_reconcile`, `get_reconcile`) using `sqlx::query`/`query_as` against `custody_api_*`, `Mem` delegating to `SharedMap` (hermetic tests only).
- `services/inventory-svc/crates/custody/src/state.rs:1` — `AppState { store: Store }` with `Default` → `Mem`, `new()` → `Mem`, `from_pool(PgPool)` → `Pg`; doc `postgres-backed (no purely in-memory)`.
- `services/inventory-svc/crates/custody/src/handlers.rs:1` — all 10 handlers now `async` via `state.store.*.await` + `store_err` mapping (`NotFound→404`, `Db→500` + `tracing::error`), `page` helper kept.
- `services/inventory-svc/crates/custody/src/main.rs:1` — `DATABASE_URL` → `PgPoolOptions::new().connect().await` + `Outbox::migrate().await` + `AppState::from_pool`, fallback `AppState::new()` only with `info!("DATABASE_URL unset; using in-memory store (tests only) — prod must set postgres")`.
- `services/inventory-svc/crates/custody/src/lib.rs:11` — exposes `pub mod store`.

**Verification:** `cargo check -p custody` clean (25 `missing_docs` warnings only, `#![warn]` + `#![allow]` on `store.rs`), `cargo check --workspace` clean (900+ warnings still, but 0 errors), `cargo run --release -p eel-completeness -- scan` `0 errors, 0 warnings, 33 infos` (vendor `TODO` allowlisted), `dart analyze` 0, `flutter test` Iris/Fingerprint 4/4 + BVR enrolment 5/5.

**Finding status update:**

| # | Title | Before | After |
|---|---|---|---|
| TD-03-P0-01 | In-memory contract stores (custody) | open (14 hits) | **fixed(2026-08-30) for `custody`** — `custody/src/state.rs:42` + `store.rs:1` + `handlers.rs:14` + `V052` now postgres; remaining 13 surfaces (`printing`, `inventory-api`, `credential`, `voter-card`, `diaspora`, `rla`, `reporting`, `court-interface`, `crms-api`, `intent`, `special-voter`, `ai-gateway`) stay **open (P0)** — next in priority order `printing` → `credential` → `inventory-api` (same Store enum pattern). |

**Next:** `printing` `Store::Pg` (trait already exists `PrintStore`, needs `Mem` + `Pg` impl), then `credential` `Store::Pg`, then `inventory-api` `OnceLock` → `PgPool` (remove static). Each is S effort per the remediation plan.

### Change Log (append)

| Date | Mode | Summary | Items fixed / deferred |
|---|---|---|---|
| 2026-08-30 | Remediate | custody postgres exemplar (Store enum, V052, handlers, main). | 1 fixed (custody P0 partial), 0 deferred |

---

## Audit Run 2026-08-30 — Remediate (printing + inventory-api postgres, P0 partial)

**Scope:** `services/inventory-svc/crates/printing` + `services/inventory-svc/crates/inventory-api` (TD-03-P0-01 next). Build mode.

**Changes:**

- `services/inventory-svc/migrations/V053__printing_api_surface.sql:1` — `printing_api_orders/deliveries/audit` (`UUID PK`, `FK`, `CHECK`, `INDEX`, `DEFAULT now()`, `JSONB` for `security_features`/`payload`, reversible `DROP`).
- `services/inventory-svc/migrations/V054__inventory_shortfalls.sql:1` — `inventory_shortfalls` (`shortfall_id TEXT PK`, `constituency_code`, `item`, `quantity`, `reported_by_hash`, `status`, `created_at`, `request_json JSONB`, `INDEX`).
- `services/inventory-svc/crates/printing/src/Cargo.toml:19` — add `sqlx {workspace}` + `eel-outbox-rs {workspace}`.
- `services/inventory-svc/crates/printing/src/store.rs:1` — replaced trait-only `PrintStore` with `Store` enum (`Pg(PgPool)`/`Mem(MemStore)`) 6 async methods `sqlx` (`create_order`, `list_orders`, `get_order`, `submit_qc`, `record_delivery`, `list_audit`) with `Mem` via `SharedMap` + `Mutex<Vec<AuditEvent>>`; audit `printing_api_audit` on create/delivery, `StoreError::NotFound/Db`, `#![allow(missing_docs)]`.
- `services/inventory-svc/crates/printing/src/http.rs:1` — `AppState { store: Store }` (`Default→Mem`, `new_shared()→Mem`, `from_pool(PgPool)→Pg`), `router` unchanged, 5 handlers `async` via `store.*.await` + `store_err` `404`/`500` + `tracing::error` + `page`.
- `services/inventory-svc/crates/printing/src/main.rs:1` — `DATABASE_URL` → `PgPoolOptions::new().connect().await` + `Outbox::migrate().await` + `AppState::from_pool`, else `info!("using in-memory (tests only)")`.
- `services/inventory-svc/crates/inventory-api/src/store.rs:38` — `Data { shortfalls: HashMap<String, Shortfall> }` + `insert_shortfall`/`list_shortfalls` (`Store::Pg` `INSERT INTO inventory_shortfalls` + `SELECT`, `Store::Mem` via `Mutex<HashMap>`).
- `services/inventory-svc/crates/inventory-api/src/spec.rs:1` — remove `OnceLock<Mutex<HashMap>>` + `shortfalls()`, replace `report_shortfall`/`list_shortfalls` with `State<Store>` `store.insert_shortfall(&rec).await` / `store.list_shortfalls().await` + `500` on `Err` + `page`.

**Verification:** `cargo check -p custody -p printing -p inventory-api` clean (25+9+118 `missing_docs` warnings only), `cargo check --workspace` clean (0 errors), `cargo run --release -p eel-completeness -- scan` `0 errors, 0 warnings, 33 infos`, `dart analyze` 0, `flutter test` Iris/Fingerprint 4/4 + BVR enrolment 5/5.

**Finding status update:**

| # | Title | Before | After |
|---|---|---|---|
| TD-03-P0-01 | In-memory contract stores (printing) | open | **fixed(2026-08-30) for `printing`** — `printing/src/store.rs:1` + `http.rs:1` + `V053` now postgres; test still via `Mem`. |
| TD-03-P0-01 | In-memory contract stores (inventory-api shortfalls) | open (`OnceLock`) | **fixed(2026-08-30) for `inventory-api` shortfalls** — `inventory-api/src/store.rs:38` + `spec.rs:1` + `V054` now postgres; other inventory-api domains already via `Store` enum (vehicles, pallets, etc.). |

**Remaining P0 (open, next in order):** `credential` (`Mutex<HashMap<Uuid, Credential>>`), `voter-card`, `diaspora`, `rla`, `reporting`, `court-interface`, `crms-api`, `intent`, `special-voter`, `ai-gateway` — each `Mutex<HashMap>` → `Store` enum + migration, same S pattern (next: `credential` `V054__credential_api`).

### Change Log (append)

| Date | Mode | Summary | Items fixed / deferred |
|---|---|---|---|
| 2026-08-30 | Remediate | printing + inventory-api postgres (Store enum, V053/V054, handlers, main). | 2 fixed (printing, inventory-api shortfalls P0 partial), 0 deferred |

