# BVR Threat Model

## Assets

- Voter PII (Tier A): national ID, biometrics, biographical data
- Voter register integrity: immutability, completeness, accuracy
- Polling station assignments (Tier B)

## Threat actors

- External attacker (internet-facing API gateway)
- Malicious insider (county coordinator, Returning Officer)
- Compromised BVR device kit

## Key mitigations

| threat | mitigation |
|---|---|
| Register tampering | Merkle root per snapshot; STH to transparency-svc; HSM countersignature |
| PII exfiltration via admin | PII masking in Django admin; purpose-of-processing logged on every reveal |
| Canary exposure | Canary rows (EC-SEC-16); canary_touched.v1 event on any read |
| Bulk export abuse | Dual-approval workflow; export audit log |
| IPRS bypass (fake IDs) | Fail-closed circuit breaker; 24h downtime = block new enrolments |
| Kit attestation failure | TPM quote verified on every ingest batch; quarantine within 60s |
| Replay attacks | Idempotency key per enrolment; UUID7 ordering |
| SQL injection | sqlx parameterized queries; RLS policies |
| Observer data isolation | RLS policy `observer_read`; no Tier A in list views |
