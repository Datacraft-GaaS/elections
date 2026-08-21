# governance-svc architecture (WP-5.1)

Five Rust axum microservices behind a shared gateway, one Django admin surface, one Tauri offline audit desktop, one Python SHANGRLA engine.

| Component | Port | Module |
|-----------|------|--------|
| eel-governance-rla | 8091 | M21 |
| eel-governance-court | 8092 | M40 |
| eel-governance-legal-audit | 8093 | M62 |
| eel-governance-reporting | 8094 | M58 |
| eel-governance-oag | 8095 | M59 |
| governance-admin (Django) | 8096 | all |
| rla-desktop (Tauri) | offline | M21 |

Data stores: PostgreSQL 17 (primary + hot standby). All writes append-only; audit shadow via V1 triggers. Compliance verdicts hashed to transparency log (sigstore Rekor). NIST beacon + HSM seed for RLA entropy. HSM-signed audit records from desktop sync on reconnect.
