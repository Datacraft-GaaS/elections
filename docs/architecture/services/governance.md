# governance-svc

## Purpose

Manages all post-election legal, audit, and governance workflows: RLA risk-limiting audits, election court evidence bundles, post-election review findings, recall petitions, referendum mechanics, by-election operations, county/national coordination, international observer coordination, parliamentary/judiciary reporting, OAG engagements, transmission legal audit, and financial forensics. It is the primary interface between the technical system and legal/constitutional processes.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M21 | rla-audit-tool | Risk-limiting audit: sample selection, SHANGRLA, NIST beacon, witness, evidence bundles |
| M40 | election-court-it-interface | Evidence bundles, petitioner access, technical witness support, eCourt interface |
| M41 | post-election-review | Findings, recommendations, parliamentary referrals, long-drip scheduler |
| M48 | recall | Recall petitions, signatory verification, threshold report, recall election trigger |
| M49 | referendum-mechanics | Referendum configuration, county/national thresholds, TLA+ verified declaration |
| M50 | by-election-operations | By-elections with register subsets, kit plan, cost report |
| M56 | county-national-coord | MoUs, county datasets, ministerial approvals, coordination incidents, SAML brokering |
| M57 | intl-observer-coord | Observer missions, briefings, reports, joint statements |
| M58 | parliamentary-judiciary-reporting | Statutory reports, testimony workspace, judiciary handover |
| M59 | oag-interface | OAG audit engagements, evidence scope, evidence packages |
| M62 | transmission-legal-audit | Transmission events, compliance verdicts, legal rule engine |
| M66 | financial-forensics | Analysis cycles, regression/DiD estimates, referral packets, replication packages |

## Key Aggregates and State Stores

- `AuditRun` (PostgreSQL): RLA run configuration with SHANGRLA parameters.
- `SampledBallot` (PostgreSQL): sampled ballot records with witness attestations.
- `EvidenceBundle` (PostgreSQL + object store): court-ready evidence packages.
- `RecallPetition` (PostgreSQL): petition records with signatory verification.
- `Referendum` (PostgreSQL): referendum configuration with TLA+ verified thresholds.
- `ByElection` (PostgreSQL): by-election instance with kit plan and register subset.
- `TransmissionEvent` (PostgreSQL, append-only): form transmission compliance log.
- `AnalysisCycle` (PostgreSQL): forensic analysis with replication packages.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | rts-svc | Forms and transmission events for legal audit |
| Upstream | transparency-svc | Reproducibility bundles for court evidence |
| Upstream | ecc-svc | Grievance referrals |
| Upstream | ai-platform-svc | Forensics referrals |
| External | eCourt | Evidence submission |
| External | NIST (beacon) | RLA entropy source |
| External | OAG | Audit engagements |
| Downstream | portal-svc | Referendum and recall calendar events |
| Downstream | training-svc | Recall and referendum training content |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.governance.rla.run.completed` | `eel.governance` | RLA audit run completed with result |
| `eel.governance.evidence_bundle.submitted` | `eel.governance` | Evidence bundle submitted to eCourt |
| `eel.governance.referendum.configured` | `eel.governance` | Referendum parameters configured |
| `eel.governance.recall.threshold.met` | `eel.governance` | Recall petition threshold reached |
| `eel.governance.transmission.verdict` | `eel.governance` | Transmission compliance verdict |
| `eel.governance.forensics.referral` | `eel.governance` | Financial forensics referral issued |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.rts.form34.published` | rts-svc |
| `eel.rts.declaration.issued` | rts-svc |
| `eel.transparency.bundle.published` | transparency-svc |
| `eel.ecc.grievance.resolved` | ecc-svc |

## Primary APIs

- `packages/openapi-contracts/governance/openapi.yaml` - RLA, review findings, referrals
- `packages/openapi-contracts/governance-rla/openapi.yaml` - RLA audit run management
- `packages/openapi-contracts/governance-court/openapi.yaml` - evidence bundles, eCourt
- `packages/openapi-contracts/governance-recall/openapi.yaml` - recall petitions
- `packages/openapi-contracts/governance-referendum/openapi.yaml` - referendum configuration
- `packages/openapi-contracts/governance-by-election/openapi.yaml` - by-elections
- `packages/openapi-contracts/governance-legal-audit/openapi.yaml` - transmission legal audit
- `packages/openapi-contracts/governance-forensics/openapi.yaml` - financial forensics
- `packages/openapi-contracts/governance-reporting/openapi.yaml` - parliamentary/judiciary reports
- `packages/openapi-contracts/governance-review/openapi.yaml` - post-election review
- `packages/openapi-contracts/governance-county-coord/openapi.yaml` - county coordination
- `packages/openapi-contracts/governance-intl-observer/openapi.yaml` - observer coordination

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| RLA sample selection P99 | less than 30 s |
| Evidence bundle submission P99 | less than 60 s |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h |

## Degradation Modes

- RLA runs degrade to offline mode with NIST beacon pre-seeded; witness attestation queues.
- eCourt API failure queues evidence bundles; operator is alerted within 5 min.
- Forensics analysis jobs are batch; no real-time degradation impact.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/incident-response.md](../../runbooks/incident-response.md)
