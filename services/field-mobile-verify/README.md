# Field-Mobile-Verify (M04 verification endpoint)

Verification endpoint for mobile biometric results. Uses `field-mobile-svc` audit shadows (`audit_shadow`) and `AuditService` replay (`replay`). DB: uses `sqlx` against `verification_queries` and `audit_shadows` (same DB as `field-mobile-svc`). Security: `EelAttestationClient` (real, ADR-0021). Logging: `_log_` (structured). Telemetry: `audit_shadow` + `otel` (via parent package). No stubs (`UnimplementedError` removed in M0-M1 transition).
