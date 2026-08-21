# crms-svc

## Purpose

Candidate and Party Registration Management: handles candidate nomination, party registration and primaries oversight, nomination fee payment, and party symbol/colour registry. It generates ballot design packages for the printing chain and maintains clearance decisions (KRA, EACC) per candidate.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M12 | crms | Candidates, nominations, clearance decisions, ballot proof |
| M45 | party-registration | Political parties, coalitions, faction disputes |
| M46 | party-primaries-oversight | Primary opt-in, results, disputes |
| M47 | nomination-fee-payment | Fee schedules, payments, waivers, refunds (Daraja) |
| M67 | party-symbol-registry | Symbol submissions, colour registration, ballot design packages |

## Key Aggregates and State Stores

- `Candidate` (PostgreSQL): nomination records with clearance statuses.
- `ClearanceDecision` (PostgreSQL): KRA/EACC tax and integrity clearances.
- `BallotProof` (PostgreSQL + HSM): signed ballot design PDF.
- `PoliticalParty` (PostgreSQL): registered parties with faction history.
- `PrimaryResult` (PostgreSQL): party primary results and dispute records.
- `Payment` (PostgreSQL): nomination fee payments with Daraja references.
- `SymbolSubmission` (PostgreSQL): party symbol and colour registration.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvr-svc | Voter register extract for candidate eligibility check |
| External | KRA | Tax compliance clearance |
| External | EACC | Integrity clearance |
| External | PPDT | Political parties tribunal decisions |
| External | Daraja / bank | Nomination fee payment |
| Downstream | inventory-svc | Ballot design package for printing |
| Downstream | portal-svc | Public candidate list |
| Downstream | transparency-svc | Campaign finance linkage |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.crms.candidate.registered` | `eel.crms` | Candidate nomination completed |
| `eel.crms.candidate.cleared` | `eel.crms` | KRA/EACC clearance received |
| `eel.crms.party.registered` | `eel.crms` | Party registration approved |
| `eel.crms.primary.result.certified` | `eel.crms` | Party primary results certified |
| `eel.crms.ballot_design.approved` | `eel.crms` | Ballot design package signed and forwarded |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvr.voter.registered` | bvr-svc |

## Primary APIs

- `packages/openapi-contracts/crms/openapi.yaml` - nomination and clearance workflow
- `packages/openapi-contracts/party/openapi.yaml` - party registration and primaries
- `packages/openapi-contracts/nomination/openapi.yaml` - nomination fee payment
- `packages/openapi-contracts/symbol/openapi.yaml` - symbol and colour registry

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Nomination submission P99 | less than 5 s |
| Clearance query P99 | less than 10 s |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h |

## Degradation Modes

- KRA/EACC API failures queue clearance requests; manual override available with operator audit trail.
- Daraja payment callback deduplication prevents duplicate payment acknowledgements.
- Ballot design approval is blocked if HSM is unavailable.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
