# abis-svc Data Catalog

Reference: ADR-0016 (data classification and retention).

## PII Fields

| Field | Table / Topic | Data Class | Retention | Encryption at Rest | Log Scrub Policy |
|---|---|---|---|---|---|
| `subject_ref` | `template_index`, events | Restricted | 10 years post-deregistration | AES-256-GCM (KEK in OpenBao) | Hashed in logs (`sha256(subject_ref)`) |
| `fingerprint_blob` | `template_index`, MinIO | Restricted | 10 years post-deregistration | AES-256-GCM; MinIO SSE-KMS | Never logged; blob_ref only |
| `iris_blob` | `template_index`, MinIO | Restricted | 10 years post-deregistration | AES-256-GCM; MinIO SSE-KMS | Never logged; blob_ref only |
| `face_blob` | `template_index`, MinIO | Restricted | 10 years post-deregistration | AES-256-GCM; MinIO SSE-KMS | Never logged |
| `match_score` | `match_results` | Sensitive | 5 years | TDE (Postgres cluster) | Suppressed from INFO-level logs |
| `probe_ref` | `match_requests` | Sensitive | 5 years | TDE | Hashed in logs |
| `gallery_ref` | `match_requests` | Sensitive | 5 years | TDE | Hashed in logs |
| `caller_id` (SVID) | `match_requests`, events | Sensitive | 5 years | TDE | Logged in full (not PII) |
| `adjudication_notes` | `adjudication_cases` | Restricted | 10 years | TDE + row-level encryption | Redacted in exports |
| `investigator_id` | `adjudication_cases` | Sensitive | 10 years | TDE | Logged in audit only |
| `tlog_leaf_hash` | `enclave_attestations` | Public | Permanent | TDE | Logged in full |
| `report_bytes` (TDX/SEV) | `enclave_attestations` | Sensitive | 5 years | TDE | Never logged raw |
| `drift_score` | `drift_alerts` | Sensitive | 90 days | TDE | Suppressed from logs |
| `outbox_payload` | `outbox` | Restricted | Deleted on ACK + 7 days | TDE | Never logged |

## Notes

- **Restricted**: requires dual-control to access in production; all queries logged to Loki.
- **Sensitive**: access logged; no unrestricted bulk export.
- **Public**: freely readable by internal services; no special controls.
- Log scrub is enforced at the `tracing` subscriber layer via the `eel-log-scrub` crate.
- Blob references (URIs like `bio-store://...`) are stored instead of raw biometric data in application logs.
- KEK rotation follows OpenBao transit re-wrap procedure (see `docs/OPERATIONS.md#kek-rotation`).
