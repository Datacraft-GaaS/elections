# vprs-svc

## 1. Overview

Voter Pre-Registration System (module M04): the public-facing intake service where citizens submit online voter pre-registration applications ahead of in-person biometric enrolment. It stores applications with hashed national-ID identity keys, issues single-use OTP verification codes bound to an application, and serves one-time QR handoff-token redemption for field kits.

Runtime: Rust 2021 (workspace `services/vprs-svc/Cargo.toml`, rust-version 1.83) built on axum 0.8 with sqlx (PostgreSQL). The workspace contains two crates: `vprs-api` (HTTP API library exposing `app()`) and `vprs-domain` (aggregates, events, OTP rules). There is no `main.rs` binary in the repository; `crates/vprs-api/src/lib.rs:7` exposes a `Router` builder that an as-yet-uncommitted binary would serve. A documented Elixir LiveView portal (`ex/vprs_web/`) is an empty skeleton (only `README.md`; `lib/vprs_web/` is empty). No port is bound anywhere in code; the documented port 8080 (`docs/CAPABILITIES.md` section 15.1) is not verified by code.

Entry points: `vprs_api::app(state)` (`crates/vprs-api/src/lib.rs:7`) building the router from `routes::router()` (`crates/vprs-api/src/routes/mod.rs:10`).

## 2. Ownership & Boundaries

Ground truth: `docs/architecture/services/vprs.md`. Note: this service has **no** `TRACEABILITY.yaml` in its folder (unlike queue-svc/inventory-svc).

| Item | Value |
|---|---|
| Modules owned | M04 vprs (application intake, IPRS validation status, application queue) |
| Data owned | `vprs_applications`, `vprs_otp_sessions` (PostgreSQL, `migrations/V001__vprs_core.sql`) |
| Upstream | IPRS (national-ID pre-validation; not wired in code yet), bvr-svc (duplicate check) |
| Downstream | biometric-edge-svc (appointment/enrolment queue), bvr-svc (enrolment) |

Documented event contract (`docs/architecture/services/vprs.md`): emits `eel.vprs.application.submitted|validated|rejected` on topic `eel.vprs`; consumes `eel.biometric_edge.enrollment.completed`. In code, only three domain event structs exist (`ApplicationSubmitted`, `ApplicationCancelled`, `HandoffRedeemed` in `crates/vprs-domain/src/events.rs`) with type strings `eel.vprs.application_submitted.v1` etc.; none are published from any handler (no outbox/Kafka wiring exists in the repo).

## 3. Repository Layout

```
services/vprs-svc/
  Cargo.toml                  # workspace: vprs-domain, vprs-api
  crates/
    vprs-api/
      src/
        lib.rs                # app(): assembles Router
        routes/mod.rs         # route table (all 7 routes)
        routes/applications.rs# create/status/cancel application
        routes/otp.rs         # OTP request + verify
        routes/handoff.rs     # QR handoff token redemption (+ local SHA-256 impl)
        routes/health.rs      # GET /health
        state.rs              # AppState: PgPool + FeatureFlags (captcha, IPRS circuit)
        error.rs              # ApiError -> HTTP status mapping
    vprs-domain/
      src/application.rs      # VPRSApplication aggregate + ApplicationStatus enum + AuditEvent
      src/events.rs           # ApplicationSubmitted / ApplicationCancelled / HandoffRedeemed
      src/otp.rs              # MAX_REQUESTS=3, LOCKOUT_MINUTES=15, OtpRequest rules
      src/ids.rs              # uuid7str()
  migrations/V001__vprs_core.sql
  deploy/service.yaml         # 7-site matrix: blue-green primary/mirror, rolling edges
  docs/CAPABILITIES.md        # capability catalogue (partly aspirational, see §7)
  ex/vprs_web/                # LiveView portal skeleton (README only; no code)
  tests/, src/, loadtest/     # placeholders (.gitkeep only)
```

## 4. Capabilities & Functions

All routes are registered in `crates/vprs-api/src/routes/mod.rs:10-26` (`router()`). Every handler is currently annotated `#[allow(dead_code)]` because no binary links them yet.

| # | Method + Route | Function | What it does | Citation |
|---|---|---|---|---|
| 1 | GET `/health` | `health_check` | Returns `{status:"ok", service:"vprs-api"}`. | crates/vprs-api/src/routes/health.rs:4 |
| 2 | POST `/vprs/v1/applications` | `create_application` | Creates an application row with status `draft`, 90-day expiry; rejects when IPRS circuit flag closed (503); maps unique-constraint violation to 409 for duplicate `id_number_hash`. Reads `Idempotency-Key` header but does not persist it. | crates/vprs-api/src/routes/applications.rs:31-73 |
| 3 | GET `/vprs/v1/applications/:id/status` | `get_status` | Returns current status string for `app_id`; 404 if unknown. | crates/vprs-api/src/routes/applications.rs:77-94 |
| 4 | DELETE `/vprs/v1/applications/:id` | `cancel_application` | Sets status `cancelled` unless already terminal (`cancelled`/`expired`/`converted`); 404 if no row affected. | crates/vprs-api/src/routes/applications.rs:99-116 |
| 5 | GET `/vprs/v1/handoff/:token` | `get_handoff` | One-time QR handoff lookup by SHA-256 token hash: 200 for `qr_issued`, 410 Gone if `converted`, 404 otherwise. | crates/vprs-api/src/routes/handoff.rs:22-51 |
| 6 | POST `/vprs/v1/otp/request` | `request_otp` | Issues a fresh 6-digit code (stored BLAKE3-hashed, TTL 600 s); 404 for cancelled/expired/converted apps; 429 when the session is locked or rate-limited. Upserts one session per app. | crates/vprs-api/src/routes/otp.rs:39-110 |
| 7 | POST `/vprs/v1/otp/verify` | `verify_otp` | Constant-time comparison of hashed code; success sets `verified=TRUE`; failure increments `attempt_count` and locks after 3 attempts for 15 min. | crates/vprs-api/src/routes/otp.rs:117-184 |

Supporting logic:

- OTP lockout/rate rules: `OtpRequest::is_locked` / `is_rate_limited` (crates/vprs-domain/src/otp.rs:14-20); constants `MAX_REQUESTS = 3`, `LOCKOUT_MINUTES = 15` (crates/vprs-domain/src/otp.rs:3-4).
- Code generation: `generate_code` derives a zero-padded 6-digit number from UUIDv4 bytes; raw value never leaves the call (crates/vprs-api/src/routes/otp.rs:188-193).
- IPRS circuit breaker flag: `FeatureFlags::is_iprs_circuit_closed` gates new submissions (crates/vprs-api/src/state.rs:24-26; checked at applications.rs:36-39).
- Application lifecycle states defined: `Draft, SubmittedIprsVerifying, IprsVerified, QrIssued, Converted, Cancelled, Expired` (crates/vprs-domain/src/application.rs:6-14).

Not implemented despite being documented: IPRS gRPC integration, OPA freeze-window middleware, rate limiter, step-up TOTP, QR generation/signing, change requests, PWA/LiveView portal (see §7 Known deferrals).

## 5. Data Model

From `migrations/V001__vprs_core.sql`:

| Table | Columns | Purpose |
|---|---|---|
| `vprs_applications` (lines 4-14) | `app_id` PK, `id_number_hash` UNIQUE NOT NULL, `status` default `draft`, `biographic_fields` JSONB, `consent_tick` BOOL, `qr_token_hash` UNIQUE, `created_at`, `updated_at`, `expires_at` | Pre-registration applications. No raw national ID stored - SHA-256 hash only (crates/vprs-domain/src/application.rs:16-17). |
| `vprs_otp_sessions` (lines 16-25) | `session_id` PK, `app_id` UNIQUE FK -> applications, `code_hash`, `attempt_count`, `verified`, `locked_until`, `expires_at`, `created_at` | One OTP session per application; hash-stored codes and lockout bookkeeping. |

Stores: sqlx PgPool only (`AppState.db`, crates/vprs-api/src/state.rs:30-33).

Events produced (defined, not emitted): `eel.vprs.application_submitted.v1`, `eel.vprs.application_cancelled.v1`, `eel.vprs.handoff_redeemed.v1` (crates/vprs-domain/src/events.rs:14,31,48). Events consumed: none in code.

## 6. Operation

Build/test (from README and Cargo layout):

```bash
cd services/vprs-svc
cargo build --release
cargo test --workspace          # unit tests incl. ids test
```

There is no runnable binary target in this workspace; serving `vprs_api::app()` requires out-of-repo wiring (not verified). The documented commands `cargo run --bin vprs-svc` and `mix phx.server` under `ex/` cannot work against the current tree (no bin target; `ex/vprs_web` has no mix.exs).

Env vars consumed in code: none (no `env::var` usage). Configuration is via `AppState::new(db)` which hardcodes feature flags captcha=true, IPRS circuit open=false (crates/vprs-api/src/state.rs:36-41).

Ports: none bound in code; documented REST 8080 / gRPC 8443 / LiveView 4000 (docs/CAPABILITIES.md section 15) are unverified.

Deploy: `deploy/service.yaml` declares image `harbor.eel.internal/eel/vprs-svc`, blue-green at primary/mirror (3 replicas each), rolling at five edge sites (2 replicas each).

Degradation: IPRS circuit-closed flag rejects new applications with 503 while status/cancel endpoints keep working (applications.rs:36-39). Documented cache-aside IPRS degradation (CAPABILITIES section 3) is not implemented.

## 7. Safeguards & Security

| Surface | Control | Citation |
|---|---|---|
| Application creation | Duplicate defence: DB UNIQUE on `id_number_hash`, mapped to 409 Conflict | applications.rs:60-67; V001 line 6 |
| Application creation | Enumeration resistance: handler doc cites constant-time anti-enumeration and idempotency key (FR-M04-021); key is read but unused | applications.rs:28,41-44 |
| OTP issuance | Rate limit 3 issues then 429; lockout 15 min; single active session per app via upsert | otp.rs:59-104; vprs-domain/src/otp.rs:3-4 |
| OTP verification | Codes stored as BLAKE3 hex hash; constant-time compare via `eel_crypto_rs::constant_time::ct_eq`; attempt consumption + auto-lock in SQL | otp.rs:83,153-154,165-181 |
| OTP secrecy | Raw code never returned or logged (module header contract) | otp.rs:1-7 |
| Handoff tokens | Token hashed before DB lookup so raw token never reaches query logs; consumed token returns 410; doc claims equivalent-work branches to avoid timing oracle | handoff.rs:17-29,39-47 |
| PII | No raw national ID persisted; audit events carry `ip_hash` (SHA-256 of source IP), not raw IP | application.rs:16-17,40-41 |
| Availability gate | IPRS circuit flag fail-closes submission endpoint | applications.rs:36-39 |

Idempotency: header parsed but not enforced (no `eel_idempotency_keys` table or check in code). Rate limiting: per-application OTP limiting only; the documented global token-bucket middleware does not exist. Audit logging: `AuditEvent` struct defined (application.rs:34-43) but nothing writes it. Validation: minimal - payload deserialization plus status guards; no field-level validation of biographic JSONB.

Tests verifying safeguards: none found. `tests/` is empty (`.gitkeep`); the only test in-tree is `uuid7str_produces_valid_uuid` (crates/vprs-domain/src/ids.rs:11-16). Documented harnesses in CAPABILITIES section 13 (contract tests, timing tests, proptest breaker tests) do not exist in the repo.

### Known deferrals

Verified against code (all paths below absent):

- USSD application submission (`*860#`) listed as follow-up in README.md:198 and CAPABILITIES section 5.3 ("planned follow-up"); confirmed not implemented.
- `crates/vprs-api/src/iprs.rs`, `src/middleware/rate_limit.rs`, `src/middleware/enum_defence.rs`, `src/qr.rs`, `src/queue.rs`, `crates/vprs-domain/src/aggregate.rs`, `src/scrubber.rs`, `deploy/authz/vprs-freeze.rego`, `proto/vprs.proto`, `loadtest/application-submit.js` - all referenced in docs/CAPABILITIES.md but not present.
- `ex/vprs_web` LiveView portal: directory skeleton only.
- Continuous register changes, soft-delete erasure (`erased_at`), QR refresh flow: documented (README Key APIs, CAPABILITIES 10.3/Appendix C) but absent.
- Handoff `sha256_hex` uses a hand-rolled SHA-256 implementation pending migration to `eel-crypto-rs` (handoff.rs:53-69 comment) - flagged in code as production gap.

## 8. Observability

- Health endpoint: GET `/health` returning static JSON (crates/vprs-api/src/routes/health.rs:4-6). No readiness endpoint.
- Logging: `tracing` used in handlers (`warn!` on circuit-closed at applications.rs:37; `info!` on OTP rate-limit/verify and handoff consumption at otp.rs:77,160 and handoff.rs:41). `tracing-subscriber` json/env-filter features declared in workspace Cargo.toml:49-51 but subscriber init lives outside the repo (no binary).
- Metrics: none implemented. Prometheus metric names listed in docs/CAPABILITIES.md Appendix E are unimplemented.
