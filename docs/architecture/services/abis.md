# abis-svc

## Purpose

Provides automated biometric identification: receives match requests from bvr-svc and biometric-edge-svc, runs 1:N deduplication and 1:1 verification against indexed templates inside a TDX/SEV confidential enclave, and returns adjudication decisions. All template data is processed exclusively within the enclave; no plaintext template leaves the enclave boundary.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M03 | abis | Template index, match requests, adjudication cases, enclave attestation |

## Key Aggregates and State Stores

- `TemplateIndex` (enclave-resident, encrypted): 1:N search index over enrolled templates.
- `MatchRequest` (PostgreSQL, metadata only): request ID, voter ID, status, score bands.
- `AdjudicationCase` (PostgreSQL): human-review queue for ambiguous match results.
- `EnclaveAttestation` (PostgreSQL): Intel TDX / AMD SEV-SNP quotes logged per session.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvr-svc | Template enrollment ingestion |
| Upstream | biometric-edge-svc | Live identification events |
| Downstream | bvr-svc | Adjudication decisions back to voter record |
| External | HSM | Key derivation for enclave session keys |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.abis.match.completed` | `eel.abis` | 1:1 or 1:N match result (score bands only, no biometric data) |
| `eel.abis.adjudication.resolved` | `eel.abis` | Human adjudication decision |
| `eel.abis.attestation.refreshed` | `eel.abis` | New enclave attestation quote logged |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvr.template.enrolled` | bvr-svc |
| `eel.biometric_edge.identification.attempt` | biometric-edge-svc |

## Primary APIs

- `packages/openapi-contracts/abis/openapi.yaml` - match requests, adjudication queue, attestation status

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| 1:1 match P99 | less than 500 ms |
| 1:N dedup batch throughput | 10K templates/min |
| Availability | 99.9% |
| RTO | 30 min (enclave re-attestation required) |
| RPO | zero (no stateful data outside enclave + PG) |

## Degradation Modes

- If the enclave fails attestation, all match requests are rejected and an operator alert fires immediately.
- Adjudication queue drains asynchronously; match requests that exceed threshold without human review are rejected.
- ABIS unavailability causes biometric-edge identification to fail closed (voter not identified).

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/incident-response-p0.md](../../runbooks/incident-response-p0.md)
