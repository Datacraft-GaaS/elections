# bvi-svc Threat Model

STRIDE analysis for M06 + M07.

## Assets

- Identification events (audit-critical, Class C)
- Kit certificates (Class C)
- Voter lookup results (Class B, PII)
- Config bundles (Class C)

## Threats

| ID | STRIDE | Threat | Control |
|---|---|---|---|
| TM-BVI-01 | Tampering | Kit sends forged identification events | Per-event kit SVID mTLS + Ed25519 signature verify; tamper event emitted on mismatch |
| TM-BVI-02 | Repudiation | Kit denies sending events | Signed events + Merkle log anchoring; sequence_counter anti-replay |
| TM-BVI-03 | Elevation | Revoked kit continues operating | Honey-mode within 5 min; revocation event propagation <= 60 s |
| TM-BVI-04 | Enumeration | Bulk BVV lookups enumerate register | Constant-time response (EC-SEC-10); per-IP + per-session rate limits; CAPTCHA step-up |
| TM-BVI-05 | Spoofing | Fake kit authenticates | SVID mTLS enforced at gateway; SPIFFE workload attestation |
| TM-BVI-06 | Info disclosure | Voter PII in logs | Telemetry rejects Tier A/B fields; log-scanner fixture in CI |
| TM-BVI-07 | DoS | Flood identification endpoint | HPA on ingest workers; Kafka backpressure; 507 on disk full |

## Mitigations pending

- TLA+ model of revocation propagation (Wave 4).
- Dudect-style timing harness for EC-SEC-11.
