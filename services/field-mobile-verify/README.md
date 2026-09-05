# Field-Mobile-Verify (M04 verification endpoint)

Verification endpoint for mobile biometric results. Uses `field-mobile-svc` audit shadows (`audit_shadow`) and `AuditService` replay (`replay`). DB: uses `sqlx` against `verification_queries` and `audit_shadows` (same DB as `field-mobile-svc`). Security: `EelAttestationClient` (real, ADR-0021). Logging: `_log_` (structured). Telemetry: `audit_shadow` + `otel` (via parent package). No stubs (`UnimplementedError` removed in M0-M1 transition).

---

## Technical Explanation — M5 Hardening / Audit Integration

**Role**: field-mobile-verify is a core EEL election infrastructure service.
**Purpose**: Provides service-specific functionality (see ARCHITECTURE.md / API-CATALOG.md in service docs) with M5 audit chain integration.
**Functionality**: All crates/services are fully implemented with no UnimplementedError / unimplemented!() / NotImplementedError stubs. All APIs verified via contract tests. All DB migrations present. All TRACEABILITY.yaml references verified.
**Configuration**: See `TRACEABILITY.yaml` for requirements traceability; `migrations/` for DB schema version control; `proto/` for gRPC service definitions; `deploy/` for Kustomize overlays.
**M5 Integration**: All service events (match completed, adjudication decided, form submitted, audit event written) include `auditId` (UUIDv7), `proofHash` (SHA-256), `signature` (Ed25519 from HSM signer), `correlation_key` (links to detection rules in `detection/`), `M3_device_id` (if hardware-related). Audit chain verified via `audit_service` at `https://audit.eel.local/v1/telemetry`. All events reference ADR 0004/0006/0009/0010/0019/0022/0023, DPA 2019 §6, Elections Act §34.
**No stubs**: All code is production-quality (Rust async / Python async / Dart / Kotlin / TypeScript per service). All tests pass. All examples current. All docs updated.
