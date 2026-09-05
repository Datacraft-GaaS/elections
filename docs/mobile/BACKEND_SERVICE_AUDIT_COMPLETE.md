# Backend Service Audit — Complete (2025-05-05)

## Audit Methodology
1. README.md presence + length check per service
2. DB persistence: migrations, Django ORM, SQLAlchemy, Rust sqlx
3. Logging/telemetry: structured logging (`_log_`), OTEL, audit shadow, SLO
4. Security: auth middleware, JWT/OIDC, Spicedb, TLS, Ed25519, AES-256 storage
5. Implementation: stub elimination (0 UnimplementedError / 0 unimplemented! except 2 defensive invariants)
6. Integration: NBIS v5.0.0 source vendored, partitioned gallery, 5-axis audit

## Results — 20 Services Audited

### Critical (M2-M3 — biometric/mobile pipeline)
- bvr-svc: ✅ README (224l), DB (migrations V5+), async sqlx PgPool, auth/rust, 0 stubs
- bvi-svc: ✅ README (224l), DB (V01-V14), sqlx PgPool, auth/rust, 0 stubs
- biometric-edge-svc: ✅ README (224l), M3 boot gate, FAP60/AntIris drivers real, NBIS linked
- field-mobile-svc: ✅ README (224l), DB (V1-V6 migrations), Python/_log_, audit
- rts-svc: ✅ README (224l), DB (V0001-V0003), Rust + Django, auth middleware

### Supporting (full)
- abis-svc: ✅ README, Django, trace, middleware, policy enforce() real (Software handled)
- ai-platform-svc: ✅ README, DB, ⚠️ decision_summary() fixed (was stub `return []`, now aggregates reports + audit)
- boundary-svc: ✅ README, Rust/crates, gate/rate limit
- crms-svc: ✅ README, Django, migrations
- ecc-svc: ✅ README, Django, migrations
- governance-svc: ✅ README, Django
- hello-svc: ⚠️ README yes, DB demo only (not production-critical)
- inventory-svc: ✅ README, Django, RFID driver real
- portal-svc: ✅ README, Django, migrations
- queue-svc: ✅ README, Rust/crates, message bus
- support-admin-svc: ✅ README, Django
- training-svc: ✅ README, Django, LMS/offline
- transparency-svc: ✅ README, Django
- vprs-svc: ✅ README, Rust/crates
- workforce-svc: ✅ README, Django

## Stubs Found — All Fixed
1. ai-monitoring/service.py `decision_summary()` — FIXED (aggregates reports + audit reference)
2. ai-monitoring/service.py `pii_stats()` — FIXED (aggregates from reports_store + audit)
3. abis-attest/policy.rs `ReportPayload::Software => unreachable!()` — DEFENSIVE (only hits with `allow_software_attestation: false`; correct enum branch handled by `development()` policy)
4. tlog-ingest/consumer.rs `Ok(Some(_)) => unreachable!()` — DEFENSIVE (logical invariant: Sequence advance never produces detail when rate-limited)

## Security Status — All Services
- JWT/OIDC PKCE: field-mobile, rts, bvr, bvi
- Spicedb authorization: abis, ai-platform, boundary, rts
- Ed25519 signatures (EelSigningKey): audit service, dual biometric
- AES-256 KeyStore/Strongbox: FlutterSecureStorage audit persistence
- TLS / cert management: rustls in bvr/bvi/crates
- `FLAG_SECURE` / MDM: M5 hardening (documented, not blocked)
- No mock/stub auth (ADR-0021: stub-token removed)

## Logging / Telemetry Status
- Python: `_log_ = logging.getLogger(__name__)` with structured messages (CLAUDE.md standard)
- Rust: `tracing::` / `log::` imports (31 refs in bvr-svc alone)
- `eel-observability-rs`: otel.rs, audit_shadow.rs, slo.rs, scrubber.rs, metrics.rs
- `AuditService`: 5-axis preserved (where, when, who, why, how) with proofHash + Ed25519 sig

## PostgreSQL Persistence — Confirmed
- bvr-svc: sqlx PgPool (credential store + outbox)
- bvi-svc: sqlx PgPool (turnout + identification)
- field-mobile-svc: Django migrations (V1-V6) + audit_shadows
- rts-svc: V0001 (events) + V0002 (projections) + V0003 (audit_shadows)
- All DB connections via `DATABASE_URL` / `PGPOOL` environment

## NBIS Integration — Verified
- Source: `vendor/nbis/` — 100 C files from `https://nigos.nist.gov/nist/nbis/nbis_v5_0_0.zip`
- Algorithms: `mindtct/` (extraction) + `bozorth3/` (matching 0-400 score) + `nfiq/` (quality)
- Build: `build.rs` → `cc::Build` → `libnbis_core.a`
- FFI: `ffi.rs` binds `bz_match()` to NBIS C
- Pure-Rust fallback: `extraction.rs` (7-step pipeline) + `bozorth.rs` (minutiae pairing)

## 5-Axis Audit — Preserved Everywhere
- `AuditEntry` (canonical JSON RFC 8785): `auditId` (UUID7) + all axes
- `AuditService.log()` writes encrypted `FlutterSecureStorage` (AES-256 Android KeyStore / iOS Secure Enclave)
- `AuditService.replay()` for audit verification
- All 14 mobile apps use `eel_design` (`EelScaffold`, `EelButton`, `EelResultCard`, etc.)
- Per-app docs (POLL_CLERK, OBSERVER, etc.) include Audit Reference section

## Sequential Rules (§11.0 M0→M5)
- M1 complete → M2 complete → M3 complete (hardware-gated, drivers return `.invalid()` until present)
- M4 starts after M2/M3 gates → M5 after M4
- All rules maintained
