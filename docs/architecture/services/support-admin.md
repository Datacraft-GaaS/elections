# support-admin-svc

## Purpose

Provides administrative support workflows that do not belong to election-day field operations: special voter facilities and IDP reassignment, intent-preservation (pre-mark validation, spoilt ballot adjudication), budget and procurement, HR and payroll for 250K election staff with M-Pesa B2C disbursements, and long-term BagIt archive management for post-election preservation.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M25 | special-voter-workflows | Special facilities, events, IDP reassignment, witness protocols, s.44A guardrails |
| M27 | intent-preservation | Pre-mark validation, spoilt ballot events, adjudication catalogue |
| M54 | budget-expenditure | Budgets, procurement, payment vouchers, CoB/OAG reports, IFMIS/PPRA |
| M55 | hr-payroll | Job applications, contracts, payroll runs, deductions, M-Pesa B2C, KRA/NSSF/NHIF |
| M69 | long-term-archive | BagIt collections, ingest jobs, preservation attestations, destruction certificates |

## Key Aggregates and State Stores

- `SpecialVoterFacility` (PostgreSQL): facility records with capacity and MoU status.
- `PreMarkValidationEvent` (PostgreSQL, append-only): on-device validation events.
- `SpoiltBallotEvent` (PostgreSQL, append-only): spoilt ballot records with images (deleted within 1 s).
- `AdjudicationEvent` (PostgreSQL): ballot adjudication decisions.
- `Budget` (PostgreSQL): budget lines and procurement records.
- `PayrollRun` (PostgreSQL): payroll runs with statutory deductions.
- `ArchiveCollection` (PostgreSQL + object store): BagIt archive collections with DOI.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | field-mobile-svc | Attendance records for payroll |
| Upstream | ecc-svc | S.44A events for special voter guardrails |
| Upstream | governance-svc | OAG evidence requests |
| External | IFMIS | Budget and payment voucher submission |
| External | PPRA | Procurement compliance reporting |
| External | M-Pesa B2C | Payroll disbursement |
| External | KRA, NSSF, NHIF | Statutory deduction returns |
| External | KNADS, DataCite | Archive accession and DOI minting |
| Downstream | governance-svc | Archive access for forensics |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.support_admin.facility.configured` | `eel.support_admin` | Special voter facility configuration confirmed |
| `eel.support_admin.spoilt_ballot.event` | `eel.support_admin` | Spoilt ballot recorded (image purged within 1 s) |
| `eel.support_admin.payroll.run.completed` | `eel.support_admin` | Payroll run completed with statutory returns |
| `eel.support_admin.archive.ingested` | `eel.support_admin` | BagIt collection ingested and attested |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.field_mobile.attendance.recorded` | field-mobile-svc |
| `eel.ecc.s44a.event` | ecc-svc |
| `eel.rts.form34.published` | rts-svc |

## Primary APIs

- `packages/openapi-contracts/support-special-voter/openapi.yaml` - special voter facilities
- `packages/openapi-contracts/support-intent/openapi.yaml` - intent preservation and adjudication
- `packages/openapi-contracts/support-budget/openapi.yaml` - budget and procurement
- `packages/openapi-contracts/support-payroll/openapi.yaml` - HR and payroll
- `packages/openapi-contracts/support-archive/openapi.yaml` - long-term archive

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Payroll run (250K staff) completion | less than 4 h |
| Spoilt ballot image deletion | less than 1 s after capture |
| Archive ingest P99 | less than 60 s per collection |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h |

## Degradation Modes

- IFMIS API failure queues payment vouchers; CoB reports are delayed not lost.
- M-Pesa B2C batch limits trigger automatic splitting; payroll completes in multiple passes.
- Archive ingest degrades to queue; no active election operations are blocked.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
