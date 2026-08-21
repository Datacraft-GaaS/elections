# bvr-svc

## Purpose

Maintains the authoritative biometric voter register: stores versioned voter records and biometric templates, ingests continuous register updates from civil registry integrations, handles voter deregistration appeals, and anchors register Merkle roots. At 30 M+ records it is the highest-throughput read store in the system and operates at 99.99% availability.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M02 | bvr-backend | Voter records, biometric templates, audit events, Merkle roots |
| M35 | cvr-continuous-register | Continuous register updates from IPRS/NRB/KRA, batch ingestion |
| M36 | voter-deregistration-appeal | Deregistration cases, appeal records, USSD/CLI adjudication |
| M53 | voter-id-card | Voter ID card batches and collection records (scope activation required) |

## Key Aggregates and State Stores

- `Voter` (PostgreSQL, versioned, 30M+ rows): canonical voter record with history.
- `BiometricTemplate` (PostgreSQL + encrypted blob store): ISO 19794-2/4 templates.
- `AuditEvent` (append-only PostgreSQL): every mutation with operator identity.
- `MerkleRoot` (PostgreSQL + transparency log): register snapshot anchor.
- `RegisterSnapshot` (PostgreSQL): periodic full-register hashes for audit.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | abis-svc | Template matching and adjudication results |
| External | IPRS | Civil registry identity data |
| External | NRB, KRA | Supplementary identity verification |
| Downstream | biometric-edge-svc | Register extracts for kit provisioning |
| Downstream | bvi-svc | Voter lookup during polling |
| Downstream | vprs-svc | Online voter status queries |
| Downstream | portal-svc | Public inspection roll |
| Downstream | transparency-svc | Register Merkle root for bundle |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.bvr.voter.registered` | `eel.bvr` | New voter record created |
| `eel.bvr.voter.updated` | `eel.bvr` | Voter record corrected or updated |
| `eel.bvr.voter.deregistered` | `eel.bvr` | Voter removed from active register |
| `eel.bvr.register.snapshot.published` | `eel.bvr` | Register snapshot with Merkle root anchored |
| `eel.bvr.template.enrolled` | `eel.bvr` | Biometric template linked to voter |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.abis.adjudication.resolved` | abis-svc |
| `eel.biometric_edge.enrollment.completed` | biometric-edge-svc |

## Primary APIs

- `packages/openapi-contracts/bvr/openapi.yaml` - voter lookup, template management, snapshot query
- `packages/openapi-contracts/cvr/openapi.yaml` - continuous register update ingest
- `packages/openapi-contracts/dereg/openapi.yaml` - deregistration and appeal
- `packages/openapi-contracts/voter-card/openapi.yaml` - card batch management (conditional)

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Voter lookup P99 | less than 50 ms |
| Template match request P99 | less than 200 ms |
| Write P99 | less than 25 ms |
| Availability | 99.99% |
| RTO | 15 min |
| RPO | zero |

## Degradation Modes

- Read path fails over to read replica if primary is under maintenance.
- Continuous register ingest queues in outbox; no voter lookup is blocked.
- If IPRS is unreachable, ingest halts gracefully; last-known record remains active.
- Template operations degrade to reference-only mode if ABIS is unavailable.

## Runbook Links

- [docs/runbooks/dr-failover-bvr-svc.md](../../runbooks/dr-failover-bvr-svc.md)
- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
