# abis-svc

Automated Biometric Identification System service. Implements M03 (biometric deduplication) for the eel election platform, covering 1:N deduplication of voter enrolments and 1:1 identity verification at polling stations.

## Architecture

Three co-deployed processes:

| Process | Binary | Role |
|---------|--------|------|
| abis-api | `abis-api` | HTTP API (Axum). Accepts match requests, serves adjudication UI, exposes reporting endpoints. Port 8080. |
| abis-matcher | `abis-matcher` | Async worker. Pulls from Kafka, runs the biometric pipeline, writes results + outbox events. Runs inside confidential-compute pods (SEV-SNP or TDX). |
| abis-vendor | Python | Thin adapter wrapping legacy or third-party SDK. Exposes a local gRPC socket consumed by abis-matcher. |

Data flows: abis-api writes a match request row and an outbox event, abis-matcher picks it up, calls abis-vendor via gRPC, scores the probe against the gallery, writes a result row and emits a match_completed event through the outbox relay.

## Enclave Attestation

abis-matcher pods require node label `eel.io/enclave=sev-snp` (or `tdx`). At startup the pod calls abis-attest to obtain an attestation quote, which is stored in `eel_enclave_attestations` and referenced on each match result via `attestation_hash`. Set `EEL_ABIS_ATTESTATION=software` in dev/kind clusters to skip hardware attestation.

## FMR/FNMR Targets

| Metric | Target |
|--------|--------|
| FMR (False Match Rate) | <= 0.001% |
| FNMR (False Non-Match Rate) | <= 0.5% |
| Lower decision threshold | 0.75 (configurable via `EEL_ABIS_LOWER_THRESHOLD`) |
| Upper decision threshold | 0.90 (configurable via `EEL_ABIS_UPPER_THRESHOLD`) |

Scores below 0.75 are `no_match`. Scores between 0.75 and 0.90 trigger an adjudication case. Scores above 0.90 are `match`.

## Adjudication SLA

Cases require a first response within 4 hours and full resolution within 24 hours. The `sla_deadline_at` column on `eel_adjudication_cases` drives alerting. Dual-control is enforced via SpiceDB: the submitting officer cannot approve their own case.

## References

- [EVENTS.md](EVENTS.md): Kafka topics and event schemas
- [THREAT-MODEL.md](THREAT-MODEL.md): STRIDE analysis
- [SLA.md](SLA.md): Availability, latency, and throughput targets
- ADR-0005: mTLS and SPIFFE SVIDs
- ADR-0015: Transactional outbox pattern
- ADR-0016: Data classification tiers
