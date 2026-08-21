# rts-svc

## Purpose

Receives, validates, stores, and publishes election result forms (Form 34A/B/C/D) transmitted from polling stations via kit devices and returning officer desktops. It performs ICR (Intelligent Character Recognition) on scanned forms, anchors Merkle roots for tamper-evidence, issues participation tokens, and drives the result declaration workflow. It is the most write-critical service in the system.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M08 | rts-device | On-kit form scanning, transmission attempts, participation token issuance |
| M09 | rts-ro | Returning Officer desktop: Form 34A/B/C entry, ICR validation, dispute cases |
| M10 | rts-backend | Form storage, Merkle anchoring, publication snapshots, ML ICR pipeline |
| M51 | result-declaration | Declaration workflow, Certificate of Election, petition-window tracker |
| M63 | ballot-serial-chain | Serial assignment, issuance receipts, spoilt ballot events, reconciliation |

## Key Aggregates and State Stores

- `Form34A/B/C/D` (PostgreSQL, append-only event-sourced): scanned and validated result forms.
- `PublicationSnapshot` (PostgreSQL): point-in-time result totals with Merkle root.
- `MerkleRoot` (PostgreSQL + transparency log): cryptographic anchor for all published forms.
- `Declaration` (PostgreSQL): signed declarations with HSM-backed certificate.
- `SerialAssignment` (PostgreSQL): ballot paper serial number ranges per station.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvi-svc | Station identification events to gate form acceptance |
| Upstream | inventory-svc | Kit configuration and ballot serial ranges |
| Downstream | transparency-svc | Signed bundle and Merkle root publication |
| Downstream | portal-svc | Public form display and inclusion proofs |
| Downstream | governance-svc | Legal audit events and petition tracking |
| Downstream | ecc-svc | Late/missing form alerts |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.rts.form34.received` | `eel.rts` | Raw form received from kit or RO desktop |
| `eel.rts.form34.validated` | `eel.rts` | Form passed ICR and integrity checks |
| `eel.rts.form34.published` | `eel.rts` | Form published to public snapshot |
| `eel.rts.merkle_root.anchored` | `eel.rts` | Merkle root committed to transparency log |
| `eel.rts.declaration.issued` | `eel.rts` | Certificate of Election issued |
| `eel.rts.serial.assigned` | `eel.rts` | Ballot serial range assigned to station |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvi.kit.session.opened` | bvi-svc |
| `eel.inventory.kit.configured` | inventory-svc |
| `eel.inventory.serial.range.allocated` | inventory-svc |

## Primary APIs

- `packages/openapi-contracts/rts/openapi.yaml` - form submission, ICR status, publication query
- `packages/openapi-contracts/declaration/openapi.yaml` - declaration workflow
- `packages/openapi-contracts/ballot-serial/openapi.yaml` - serial chain

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Write P99 latency (kit to ACK) | less than 25 ms |
| Concurrent sessions | 60,000 (ADR-0023) |
| Availability | 99.99% |
| Merkle anchor latency P99 | less than 60 s |
| RTO | 15 min |
| RPO | zero (Kafka MirrorMaker 2 + outbox) |

## Degradation Modes

- If PostgreSQL T1 primary is unreachable, writes fail-fast; outbox relay retries from kit indefinitely.
- ICR pipeline degrades to manual RO entry mode; forms remain in `pending-icr` state.
- Publication halts if HSM is unreachable; results queue without public disclosure.
- Merkle anchoring retries with exponential backoff; a stale-anchor alert fires after 5 min.

## Runbook Links

- [docs/runbooks/dr-failover-rts-svc.md](../../runbooks/dr-failover-rts-svc.md)
- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/event-replay.md](../../runbooks/event-replay.md)
