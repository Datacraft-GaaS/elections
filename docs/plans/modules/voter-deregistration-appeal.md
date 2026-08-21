# M36 voter-deregistration-appeal

| field | value |
|---|---|
| M-number | M36 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M36-voter-deregistration-appeal.md |

## Purpose

Provide the statutory workflow for removing a voter from the register and adjudicating that voter's (or their representative's) appeal against removal per Elections Act s.8A; enforce ODPC-compliant data-minimisation on deregistered records; publish a quarterly deregistration report on the public portal (M11); distinct from M35 which proposes removals based on agency feeds.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Case Officer Console | Phoenix LiveView | services/bvr-svc/ex/deregistration_web/ |
| Adjudicator Workspace | Phoenix LiveView | services/bvr-svc/ex/deregistration_web/ |
| Commissioner Sign-Off Panel | Django | services/bvr-svc/django/gazette_admin/ |
| Voter Notice Page + Appeal Form | Astro | services/bvr-svc/astro/deregistration-public-site/ |
| ODPC Compliance Log | Django | services/bvr-svc/django/gazette_admin/ |
| API | Rust (axum) | services/bvr-svc/crates/bvr-api/ |

Languages: EN + SW; notices in county-detected Kenyan language; appeal intake in EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| DeregistrationCase | id, voter_id, m35_event_id, status (NOTICED/APPEAL_FILED/UNDER_REVIEW/ADJUDICATION/OVERTURNED/CONFIRMED/DEREGISTERED/EXPIRED_NO_APPEAL), notice_dispatched_at, notice_deadline (dispatched + 30 days), appeal_id, officer_id, officer_recommendation, adjudicator_id, adjudicator_decision, adjudicator_decision_at, adjudicator_notes, data_minimisation_at, data_minimisation_officer, created_at | A | T1-audit |
| AppealRecord | id, case_id, intake_channel (VERIFIER_APP/WEB/USSD/IN_PERSON), appellant_id_hash (SHA-256 of National ID), representative_auth_file_id, grounds (IDENTITY_ERROR/ALIVE_AND_PRESENT/DATA_MATCH_ERROR/FAMILY_DISPUTE/OTHER), grounds_detail, supporting_doc_ids, language (BCP-47), filed_at, hearing_date | A | T1-audit |
| QuarterlyDeregistrationReport | id, period_start, period_end, county_breakdown, total_proposed, total_noticed, total_appealed, total_overturned, total_confirmed, report_pdf_hash, merkle_inclusion_proof, commissioner_signed_at, published_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/bvr/deregistration-appeal.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/deregistration/cases | internal gRPC | SVID mTLS (M35_INTERNAL) | yes | create deregistration case from M35 approval |
| GET /v1/deregistration/cases | 2 | JWT+SpiceDB (IEBC_OFFICER) | n/a | triage queue |
| GET /v1/deregistration/cases/{id} | 2 | JWT+SpiceDB (IEBC_OFFICER) | n/a | case detail |
| PATCH /v1/deregistration/cases/{id} | 2 | JWT+SpiceDB (IEBC_OFFICER) | yes | update triage recommendation |
| POST /v1/deregistration/cases/{id}/appeals | 1 | public + case reference | yes | file appeal (voter or representative) |
| GET /v1/deregistration/cases/{id}/appeals/{aid} | 2 | JWT+SpiceDB (IEBC_ADJUDICATOR) | n/a | get appeal record |
| POST /v1/deregistration/cases/{id}/decision | 2 | JWT+SpiceDB (IEBC_ADJUDICATOR) | yes | record adjudicator decision |
| POST /v1/deregistration/cases/{id}/minimise | 2 | JWT+SpiceDB (IEBC_OFFICER) | yes | execute data-minimisation action |
| GET /v1/deregistration/reports | 1 | public | n/a | list quarterly deregistration reports |
| GET /v1/deregistration/reports/{id} | 1 | public | n/a | download report PDF + Merkle proof |
| POST /v1/deregistration/reports/{id}/sign | 2 | JWT+SpiceDB (IEBC_COMMISSIONER) + HSM | yes | Commissioner sign and publish; 2/quarter |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.deregistration_notice_sent.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.appeal_filed.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.voter_overturned.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | exactly-once |
| eel.bvr.voter_deregistered.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | exactly-once |
| eel.bvr.data_minimisation.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | exactly-once |
| eel.bvr.removal_approved.v1 | consumes | eel.bvr.cvr.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M35 CVR | internal | integrations/cvr/ | yes | proposed-removal event source |
| M02 BVR Back-End | internal | integrations/bvr/ | yes | voter record reinstatement or final-status update |
| M03 ABIS | internal | integrations/abis/ | yes | soft-delete or lift of biometric soft-hold |
| M13 Public Verifier App | internal | integrations/verifier/ | yes | appellant self-service portal |
| M11 Public Portal | internal | integrations/portal/ | yes | quarterly report publication |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | aggregate deregistration metrics |
| M40 Election Court IT Interface | internal | integrations/court/ | yes | adjudicator decisions for petitioners |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | quarterly report anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes | Commissioner countersigning |
| Safaricom / Africa's Talking | I08 | integrations/ussd-gateway/ | yes | *IEBC# USSD appeal intake |
| ODPC | external | integrations/odpc/ | yes (stub) | DPIA register lookup; compliance receipt submission |

## ML and AI

- None.

## Dependencies

- Modules: M02, M03, M11, M13, M15, M35, M40
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

1. FR-M36-001: deregistration notice (PDF) for every M35-approved removal; dispatch via SMS link + postal address + M13 push; notice period 30 days per Elections Act s.8A; dispatch within 1 hour of M35 approval
2. FR-M36-002: appeal intake via 4 channels: M13 Public Verifier App, web form, *IEBC# USSD, in-person IAB office; creates case record with uuid7 appeal_id, intake channel, appellant identity, grounds, supporting documents
3. FR-M36-003: case-officer triage; categorises appeal as IDENTITY_ERROR/ALIVE_AND_PRESENT/DATA_MATCH_ERROR/FAMILY_DISPUTE; requests additional evidence if needed; recommends UPHOLD_REMOVAL or OVERTURN
4. FR-M36-004: adjudicator hearing; full case file view (officer recommendation, evidence, M35 source summary; no raw CRS/NRB data); written decision within 14 days; decision final for administrative purposes; voter retains court petition right
5. FR-M36-005: OVERTURN path; M35 event reversed; ABIS soft-hold lifted; voter record REINSTATED; voter notified within 24 hours
6. FR-M36-006: UPHOLD path; record moved to DEREGISTERED; ABIS template soft-deleted (recoverable for 7 years); contact data purged per DPA 2019 s.26; voter notified with right-to-petition advisory
7. FR-M36-007: no-appeal expiry; after 30-day notice period with no appeal, case officer confirms; record moved to DEREGISTERED after Commissioner countersign
8. FR-M36-008: ODPC compliance; before final deregistration, system checks DPIA register; logs data-minimisation action (contact/address/photo purge) with officer_id, timestamp, DPA basis code; compliance receipt retained 7 years
9. FR-M36-009: quarterly deregistration report; aggregated per county (proposed, noticed, appealed, overturned, confirmed); signed by IEBC Commissioner; published on M11
10. FR-M36-010: quarterly report PDF hash anchored in P05 Merkle chain at publication
11. FR-M36-011: multi-language; notices in EN + SW + county-detected language; appeal intake supports EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya
12. FR-M36-012: deadline tracking; 30-day notice period and 14-day adjudication SLA enforced; supervisor alert when deadline at risk
13. Legal representative appeal: uploads power of attorney + appeal grounds; system accepts authorisation
14. Raw CRS/NRB source documents never stored in appeal case file; adjudicator sees only IEBC-generated summary
15. 7-year audit-hold flag prevents irreversible data purge during appeal window; officer confirmation required for minimisation action
16. WCAG 2.2 AA: voter notice page, appeal intake form, status tracker

## Acceptance criteria

1. AC-M36-01: notice dispatch within 1 hour of M35 approval; dispatch timestamp measurement
2. AC-M36-02: appeal acknowledgement <= 5 minutes from submission; case-creation timestamp
3. AC-M36-03: adjudication SLA compliance >= 95% within 14 days; SLA tracking log
4. AC-M36-04: data-minimisation completion 100% within 30 days of confirmed deregistration; ODPC audit log
5. AC-M36-05: quarterly report published within T+7 of quarter-end; publication timestamp
6. AC-M36-06: 100% quarterly report Merkle anchors; Merkle reconciliation
7. AC-M36-07: zero PII in public report; automated PII scan on report before publication

## Applicable edge cases

**identity**
- EC-IDN-04: voter unreachable at registered contact; multi-channel notice (SMS + postal + M13 push); public gazette as constructive notice

**data**
- EC-DAT-12: premature data-minimisation action requested; 7-year audit-hold flag blocks irreversible purge during appeal window; officer confirmation required
- EC-DAT-16: adjudicator attempts to access raw CRS/NRB source data; access denied; only IEBC-generated summary accessible

**legal**
- EC-LEG-01: 30-day appeal deadline enforced strictly; system rejects late appeals; officer can escalate to Commissioner for extraordinary extension
- EC-LEG-06: adjudicator decision challenged in court; written decision required; Merkle-anchored evidence trail admissible

**operations**
- EC-OPS-04: high appeal volume overwhelming adjudicators; case officer pre-filters weak appeals; IEBC can assign additional commissioners
- EC-OPS-06: M35 event queue backlog; case creation does not block notice dispatch; parallel queues

**scale**
- EC-SCL-01: 5,000 concurrent appeal submissions during large post-census batch; back-end worker auto-scales; appeal intake p99 <= 500 ms

**human**
- EC-HUM-06: WCAG 2.2 AA on notice page, appeal form, status tracker; screen reader support
- EC-HUM-08: 7-language appeal intake; county-detected language for notices

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | notice-deadline calculator, appeal-grounds classifier, 7-year hold guard, data-minimisation trigger | nextest | M36-UNIT- |
| property | deadline arithmetic invariants, appeal idempotency | proptest | M36-PROP- |
| contract | M35 case creation event, M02 reinstatement, M03 ABIS hold lift, M11 report publication | Pact | M36-PACT- |
| integration | M35 event to case creation; appeal via USSD webhook; OVERTURN reinstatement; quarterly report publication | testcontainers | M36-INT- |
| e2e | CRS-based removal; voter appeals via web; officer recommends OVERTURN; adjudicator confirms; voter reinstated | Playwright | M36-E2E- |
| security | access another voter's case; access raw CRS data via appeal API; verify hold prevents premature purge | ZAP + cargo-fuzz | M36-SEC- |
| a11y | WCAG 2.2 AA on notice page, appeal form, status tracker | axe-core + screen reader | M36-A11Y- |
| load | 5,000 concurrent appeals; p99 <= 500 ms | k6 | M36-LOAD- |
| chaos | M35 event queue backlog; notice dispatch unblocked | Chaos Mesh | M36-CHAOS- |

Tags: `#[trace("FR-M36-004","AC-M36-03","EC-LEG-06")]`

## Runbooks and dashboards

- `docs/runbooks/bvr-svc-degradation.md` - extend with deregistration appeal paths
- `docs/runbooks/adjudicator-sla-breach.md` - new runbook; alert escalation to IEBC Commissioner; emergency adjudicator assignment
- `docs/runbooks/data-minimisation-audit.md` - new runbook; ODPC audit trail retrieval; breach notification steps
- Grafana dashboard: `platform/observability/bvr-svc/deregistration-dashboard.json` (named `eel-iems-36`)
- SLO alerts: `platform/observability/bvr-svc/deregistration-slo-alerts.yaml` (alert ruleset `iems-36-alerts`; notice 1 h, adjudication 14 d)

## Open questions

- ODPC DPIA registration: Datacraft prepares draft DPIA at contract start; IEBC submits within 30 days of award; delays risk blocking final deregistrations.
- *IEBC# USSD short-code: same gateway as *400# or separate? Confirm with Safaricom; short-code reservation has 6-8 week lead time.
- Adjudicator pool size: IEBC must confirm number of commissioners available as adjudicators; scalability under post-census catch-up batches depends on this.
- Court petition right advisory text: legal team must approve EN + SW + 7-language versions of the right-to-petition advisory before go-live.
