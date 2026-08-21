# transparency-svc

## Purpose

Publishes the reproducibility bundle and campaign-finance disclosures so that any observer can independently verify election results and scrutinise candidate funding. The service owns the audit trail that makes tallying auditable end-to-end: signed form bundles, Merkle roots, participation tokens, and bug-bounty evidence packages. It also manages statutory campaign-finance reporting against CBK/FRC thresholds.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M14 | reproducibility-bundle | Reproducible result bundles, participation tokens, bug-bounty disclosure |
| M22 | campaign-finance | Candidate/party finance disclosures, cash aggregation, CBK/FRC interface |

## Key Aggregates and State Stores

- `Bundle` (PostgreSQL, append-only): signed Form 34* bundles with Merkle root and HSM signature.
- `ParticipationToken` (PostgreSQL): per-voter anonymous tokens proving ballot counted.
- `BugBounty` (PostgreSQL): researcher evidence submissions and triage status.
- `CandidateDisclosure` (PostgreSQL): campaign finance declarations.
- `CashAggregate` (Redis cache, refreshed per reporting window).

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | rts-svc | Signed Form 34* and publication snapshots |
| Upstream | bvr-svc | Voter register extract for token issuance |
| Upstream | crms-svc | Candidate list for finance linkage |
| Downstream | portal-svc | Publishes bundle download links |
| Downstream | ecc-svc | Bug-bounty escalation events |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.transparency.bundle.published` | `eel.transparency` | New reproducibility bundle signed and available |
| `eel.transparency.token.issued` | `eel.transparency` | Participation token issued to voter |
| `eel.transparency.finance.disclosure.submitted` | `eel.transparency` | Campaign finance declaration received |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.rts.form34.published` | rts-svc |
| `eel.rts.merkle_root.anchored` | rts-svc |
| `eel.crms.candidate.registered` | crms-svc |

## Primary APIs

- `packages/openapi-contracts/transparency/openapi.yaml` - public bundle query and token verification
- `packages/openapi-contracts/party/openapi.yaml` - party-portal finance submission

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Bundle publication P99 latency | less than 30 s from final Merkle anchor |
| Token query P99 | less than 200 ms |
| Availability | 99.9% (non-write-path) |
| RTO (DR failover) | 15 min |
| RPO | zero (event log replicated) |

## Degradation Modes

- If rts-svc event stream is delayed, bundle publication queues; a stale-bundle alert fires after 10 min.
- If HSM is unreachable, signing halts and an operator alert fires. Unsigned bundles are never published.
- Campaign-finance read path continues from read replica if primary is unavailable.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/incident-response.md](../../runbooks/incident-response.md)
