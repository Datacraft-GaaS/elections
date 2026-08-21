# training-svc Threat Model

## Trust boundary summary

training-svc is routine availability (99.5%); accreditation and T-30 to T-1 LMS window 99.9%.
All data classified per ADR-0006 (B/C unless noted as A in entity specs).
HSM signing required for certificates and badges (P06).

## Top threats

| Threat | Mitigation |
|---|---|
| Unreviewed content published to M13 feed | SpiceDB COMMISSION_DIRECTOR gate on approve action; fact-check required before approval queue |
| Badge forgery (QR tampering) | HSM-signed JWT in QR; scanner verifies signature offline with embedded IEBC public key |
| CRL stale (revoked badge accepted) | CRL TTL 5 min; scanner alerts if CRL age > 30 min; CRL signed by P06 |
| PHI in medical incident | no-phi schema validator rejects patient name/ID fields; Semgrep CI rule enforced |
| Assessment answer tampering | Score computed server-side only; client submits raw answers |
| Certificate spoofing | Certificate PDF SHA-256 hash stored on WorkerCertificate; HSM signature required |
| Reviewer accesses out-of-queue content | SpiceDB scopes reviewer to assigned review queue only |
| Worker accesses another worker certificate | SpiceDB GET /certificates/:id scoped to self or HR role |
| NPS advisory data retained | Session-cleared; no NPS fields in persisted MediaAccreditation model |
