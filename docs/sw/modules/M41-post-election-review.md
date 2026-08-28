# M41 — Post-Election Review

**Module ID:** M41
**Name:** Post-Election Review and Recommendation Tracking Platform
**Runs on:** Central servers (Kenyan cloud)
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Capture structured lessons-learned from every stakeholder category (IEBC departments, IAB, KEOG, ELOG, accredited observers, media, and citizens) after each election cycle. Track each finding through a lifecycle from submission to recommendation status, parliamentary response, and IEBC-Chair sign-off. Feed confirmed recommendations back into next-cycle planning to close the loop. Addresses Strategy §XXI.C.20 (long-drip attack mitigation) by ensuring systemic weaknesses are formally recorded and assigned to responsible owners before institutional memory dissipates.

Strategy reference: §XXI.C.20 (long-drip attacks), §XXI (post-election review cycle).

## 2. User Personas

- **IEBC Department Lead** — submits lessons-learned from their department; owns assigned recommendations
- **IAB Commissioner** — submits independent assessment; reviews IEBC draft responses to IAB findings
- **KEOG Observer Coordinator** — submits aggregated KEOG field observations; endorses or disputes IEBC response
- **ELOG / EU EOM / Domestic Observer** — submits structured observation report; views published recommendations
- **Media Partner** — submits media-access and transparency incident reports; views public-facing recommendation tracker
- **Citizen Contributor** — submits experience report via M13 or web form; views public summary
- **IEBC Chair** — signs off on final consolidated recommendation set; assigns departmental owners
- **Parliament Liaison (Clerk of National Assembly)** — receives recommendation set requiring legislative action; submits Parliament response
- **Next-Cycle Planning Lead** — pulls confirmed recommendations into next-cycle planning module
- **ODPC DPO (Datacraft)** — ensures citizen submission data handled per DPA 2019

## 3. Functional Requirements

- **FR-M41-001** Lessons-learned intake: structured intake form with: submitter category, election cycle reference, finding type (TECHNICAL / PROCESS / LEGAL / SECURITY / ACCESSIBILITY / COMMUNICATION), severity (HIGH/MEDIUM/LOW), description, evidence references (optional blob or URL), recommended action.
- **FR-M41-002** Multi-channel intake: (a) internal portal for IEBC departments and IAB; (b) observer portal (credentialed KEOG/ELOG/EU EOM); (c) public form via M13 and web (citizen and media); (d) structured upload (CSV/JSON) for bulk observer report ingestion.
- **FR-M41-003** Finding deduplication: system identifies semantically similar findings across submitter categories using keyword matching + optional M28 AI clustering; flags duplicates for consolidation by IEBC analyst; analyst confirms or keeps separate.
- **FR-M41-004** Recommendation lifecycle: each finding transitions through: SUBMITTED → TRIAGED → DRAFT_RECOMMENDATION → IEBC_RESPONSE → STAKEHOLDER_REVIEW → FINALISED → ASSIGNED → IN_PROGRESS → PARLIAMENT_REFERRAL (if legislative action needed) → CLOSED. State transitions require authorised role actions and are immutable once made.
- **FR-M41-005** IEBC departmental response: for each recommendation assigned to a department, the Department Lead provides: response type (ACCEPTED / ACCEPTED_WITH_MODIFICATION / DECLINED / DEFERRED), response rationale, proposed implementation date, required resources.
- **FR-M41-006** Stakeholder review: after IEBC response is drafted, submitter (IAB, KEOG, ELOG) has 30 days to endorse or dispute the response. Disputes flagged to IEBC Chair for resolution.
- **FR-M41-007** IEBC Chair sign-off: consolidated recommendation set submitted to IEBC Chair for final sign-off; Chair can approve, return to department, or escalate to Commissioner panel. Signed with HSM credential. Sign-off triggers: assignment of owner, implementation date, and escalation path.
- **FR-M41-008** Parliament referral: recommendations requiring legislative action (Elections Act amendments, IEBC Act amendments) are packaged as a formal referral to the Clerk of the National Assembly with the IEBC Chair's signature. Parliament Liaison records parliamentary response (ADOPTED / REJECTED / DEFERRED) and legislative reference.
- **FR-M41-009** Long-drip attack register: a dedicated register within the platform for findings categorised as slow-burn systemic threats (§XXI.C.20). Long-drip findings have mandatory 6-month review intervals even after closure; if not addressed before the next election cycle, they auto-escalate to Commissioner panel.
- **FR-M41-010** Next-cycle integration: at the start of the next planning cycle, the Next-Cycle Planning Lead pulls all ASSIGNED or IN_PROGRESS recommendations into the planning register. The platform generates a one-page handoff brief per recommendation with owner, deadline, and implementation status.
- **FR-M41-011** Public recommendation tracker: a public-facing view (M11 portal) showing each recommendation number, finding type, severity, IEBC response, current status, and responsible department. No PII. Updated weekly.
- **FR-M41-012** Analytics and reporting: aggregate dashboards showing: findings by category, severity, submitter type; response rates; open vs closed; parliament referrals; by-election cycle comparison.
- **FR-M41-013** Multi-language: citizen intake form available in EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya. Internal portal in EN + SW.

## 4. Non-Functional Requirements

- **NFR-M41-001** Platform availability: >= 99% from T+1 through T+180 (six months post-election).
- **NFR-M41-002** Intake response: p99 <= 800 ms for form submission.
- **NFR-M41-003** DPA 2019: citizen submissions may contain personal data; lawful basis s.30(1)(b) (legal obligation — IEBC Act §7 reporting requirement) and s.30(1)(a) (consent for identified submissions); citizen submissions stored encrypted; option to submit anonymously; retention 7 years.
- **NFR-M41-004** WCAG 2.2 AA: citizen public form, public recommendation tracker, observer portal.
- **NFR-M41-005** Deduplication: M28 clustering returns candidate duplicates within 10 seconds of each new finding submission.
- **NFR-M41-006** Audit trail: every state transition, IEBC response, Chair sign-off, and Parliament response immutably logged with actor, timestamp, and role.
- **NFR-M41-007** Scalability: handle up to 20,000 citizen submissions within T+7 post-election.
- **NFR-M41-008** Long-drip alert reliability: 6-month review interval trigger must not be missed; implemented as durable scheduled job with dead-letter alerting.

- **NFR-41-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-41` (replace 41 with the module number); alert ruleset ID `iems-41-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As an IEBC IT Department Lead, I submit 12 structured lessons-learned findings from election day; I track each one through to IEBC Chair sign-off and see my department's assigned recommendations with implementation deadlines.
2. As an IAB Commissioner, I submit the IAB independent post-election assessment as a bulk JSON upload; I receive a submission reference and monitor how IEBC responds to each IAB finding.
3. As a citizen, I submit a report via M13 about a long queue at my polling station; I can optionally remain anonymous; I view the public recommendation tracker to see if a queue-management improvement is being tracked.
4. As an IEBC Analyst, I see two structurally similar findings from ELOG and KEOG about biometric scanner failures; I flag them as duplicates; the system consolidates them into one recommendation.
5. As an IEBC Chair, I review the consolidated recommendation set, approve 47 of 50 recommendations, return 3 to departments for revision, and sign off with my HSM credential; the signed set is published on M11.
6. As a Parliament Liaison (Clerk of the National Assembly), I receive the formal package of 4 recommendations requiring Elections Act amendments; I record Parliament's response (3 adopted, 1 deferred) with the bill reference numbers.
7. As a Next-Cycle Planning Lead at cycle start, I pull all 12 IN_PROGRESS recommendations into the 2032 planning register and receive a one-page brief per recommendation.
8. As a media partner, I submit 3 incident reports about information-access denials on election day; I view the public tracker to confirm they were triaged and a recommendation was raised.
9. As an IEBC Security Lead, I review the long-drip attack register 6 months after the election and see 2 items that have not yet been assigned to an owner; I escalate to the Commissioner panel.
10. As an ELOG observer coordinator, I dispute IEBC's response to my finding about Form 34A posting compliance; the dispute is escalated to the IEBC Chair for resolution within 14 days.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime, date

class LessonsLearningFinding(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    election_cycle: str             # e.g. "2027-KE-GE"
    submitter_category: Literal[
        "IEBC_DEPT","IAB","KEOG","ELOG","EU_EOM","DOMESTIC_OBSERVER",
        "MEDIA","CITIZEN"
    ]
    submitter_id: str | None        # None if anonymous citizen
    finding_type: Literal[
        "TECHNICAL","PROCESS","LEGAL","SECURITY","ACCESSIBILITY","COMMUNICATION"
    ]
    severity: Literal["HIGH","MEDIUM","LOW"]
    description: str
    evidence_refs: list[str]
    recommended_action: str
    is_long_drip: bool              # §XXI.C.20 classification
    duplicate_of: str | None        # FK to consolidated finding
    status: Literal[
        "SUBMITTED","TRIAGED","DRAFT_RECOMMENDATION","IEBC_RESPONSE",
        "STAKEHOLDER_REVIEW","FINALISED","ASSIGNED","IN_PROGRESS",
        "PARLIAMENT_REFERRAL","CLOSED"
    ]
    submitted_at: datetime
    created_at: datetime

class Recommendation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    finding_ids: list[str]          # consolidated from 1 or more findings
    recommendation_text: str
    iebc_response_type: Literal[
        "ACCEPTED","ACCEPTED_WITH_MODIFICATION","DECLINED","DEFERRED"
    ] | None
    iebc_response_rationale: str | None
    owner_dept_id: str | None
    implementation_date: date | None
    requires_legislation: bool
    parliament_referral_id: str | None
    chair_signed_at: datetime | None
    chair_signed_by: str | None
    next_cycle_handoff_at: datetime | None
    last_long_drip_review: datetime | None

class ParliamentReferral(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    recommendation_ids: list[str]
    referral_package_hash: str
    chair_signed_at: datetime
    clerk_received_at: datetime | None
    parliament_response: Literal["ADOPTED","REJECTED","DEFERRED"] | None
    bill_reference: str | None
    parliament_response_at: datetime | None

class LongDripAlert(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    finding_id: str
    review_due_at: datetime         # every 6 months post-closure
    reviewer_id: str | None
    reviewed_at: datetime | None
    escalated_to_commissioner: bool
    escalated_at: datetime | None
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/review/findings | Submitter JWT or Public (anon) | 100/hr per IP | Submit finding / lessons-learned |
| POST | /v1/review/findings/bulk | OBSERVER JWT | 10/day | Bulk CSV/JSON finding import |
| GET | /v1/review/findings | IEBC_ANALYST JWT | 200/hr | List findings (filterable by type, status) |
| PATCH | /v1/review/findings/{id} | IEBC_ANALYST JWT | 100/hr | Triage: deduplicate, categorise, escalate |
| GET | /v1/review/recommendations | IEBC_OFFICER JWT | 200/hr | List recommendations |
| PATCH | /v1/review/recommendations/{id} | DEPT_LEAD JWT | 100/hr | Submit IEBC departmental response |
| POST | /v1/review/recommendations/{id}/sign | IEBC_CHAIR HSM | 5/hr | Chair sign-off on recommendation |
| GET | /v1/review/public | Public | 500/min | Public recommendation tracker (no PII) |
| POST | /v1/review/parliament-referrals | IEBC_CHAIR HSM | 2/cycle | Create and sign Parliament referral package |
| PATCH | /v1/review/parliament-referrals/{id} | PARLIAMENT_LIAISON JWT | 10/hr | Record Parliament response |
| GET | /v1/review/long-drip | IEBC_SECURITY JWT | 50/hr | Long-drip register with review status |
| POST | /v1/review/long-drip/{id}/review | IEBC_SECURITY JWT | 20/hr | Record 6-month review |
| GET | /v1/review/next-cycle-handoff | PLANNING_LEAD JWT | 10/cycle | Pull ASSIGNED/IN_PROGRESS recs for next cycle |

## 8. Integration Points

- **M13 Public Verifier App** — citizen finding intake channel
- **M11 Public Portal** — public recommendation tracker
- **M15 IAB Dashboard** — IAB findings submission; aggregate analytics
- **M28 AI Monitoring Stack** — finding deduplication clustering (optional)
- **P05 Merkle Chain** — Chair sign-off and Parliament referral package anchoring
- **P06 HSM** — Chair signing
- **Parliament eCitizen / Clerk portal** (external) — referral delivery

## 9. UI/UX Key Screens

1. **Citizen Intake Form (M13 + Web, WCAG 2.2 AA)** — finding type picker with icons, severity self-select, free-text description, evidence upload (optional), anonymous toggle, language selector; submits in EN or 6 Kenyan languages.
2. **IEBC Analyst Console** — kanban: SUBMITTED / TRIAGED / IN_RECOMMENDATION; finding cards with severity badges; duplicate-candidate flags; bulk triage tools.
3. **Recommendation Detail** — consolidated finding list, draft recommendation text, department response form, stakeholder-review status, Chair sign-off button.
4. **Public Recommendation Tracker (M11 integration, WCAG 2.2 AA)** — filterable table: recommendation ID, finding type, severity, IEBC response summary, status, responsible department; last-updated timestamp.
5. **IEBC Chair Sign-Off Panel** — recommendation list with status summary; chair notes field; HSM sign button; cannot proceed if any item is in STAKEHOLDER_REVIEW with an unresolved dispute.
6. **Parliament Referral Package** — list of legislative recommendations; referral letter preview; Chair sign and dispatch; tracking of Parliament response with bill reference.
7. **Long-Drip Register** — list of §XXI.C.20 findings; next-review-due countdown; owner; escalation button; review history per finding.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Post-election reporting obligation | IEBC Act 2011 §7(1)(j) (annual report to National Assembly) |
| Recommendation to Parliament | IEBC Act 2011 §9(2) (Commissioner recommendations to Parliament) |
| Observer participation in review | Elections Act 2011 §34 (observer accreditation and reporting) |
| Citizen participation | Constitution Art 10 (participatory governance) |
| Data protection — citizen submissions | DPA 2019 §30(1)(a) (consent) and §30(1)(b) (legal obligation) |
| Anonymous reporting protection | DPA 2019 §29 (sensitive data); Whistleblower Protection Act 2022 |
| Electronic records admissibility | Electronic Transactions Act 2007 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Findings intake open within | T+2 | Platform launch timestamp | IEBC Chair |
| IEBC departmental response to all HIGH findings | 100% within T+30 | Response timestamp | IEBC Chair |
| IEBC Chair sign-off on consolidated set | <= T+60 | Sign-off timestamp | IEBC Commissioner |
| Public recommendation tracker published | <= T+7 | Publication timestamp | IEBC Chair |
| Parliament referral package submitted | <= T+90 | Referral dispatch timestamp | Clerk of National Assembly |
| Long-drip 6-month review compliance | 100% | Alert log | IEBC Security Lead |
| Next-cycle handoff completeness | 100% of ASSIGNED/IN_PROGRESS recs | Handoff report | Next-Cycle Planning Lead |

## 12. Test Strategy

- **Unit:** recommendation lifecycle state machine; long-drip 6-month interval scheduler; deduplication candidate scorer; Chair sign-off completeness gate.
- **Integration:** citizen intake via M13; IAB bulk upload; M28 clustering call; Chair sign-off to Merkle anchor; Parliament referral dispatch.
- **E2E:** simulate post-election: submit 30 findings across 5 submitter types; triage; draft recommendations; departmental responses; Chair sign-off; public tracker update; verify Merkle anchor.
- **Security:** anonymous citizen submits finding — verify no PII stored; attempt to modify closed recommendation (must fail — immutable state); attempt to access non-public findings without role (must fail).
- **Accessibility:** WCAG 2.2 AA on citizen intake form, public tracker, and observer portal (axe-core + manual + 3-language test).
- **Load:** 20,000 citizen submissions within T+7; verify no queue starvation on deduplication.
- **Long-drip:** advance system clock 6 months; verify alert fires for un-reviewed long-drip findings; verify escalation triggers on second missed review.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Platform live (internal) | 2027-08-12 | T+2: IEBC departments begin submitting |
| Citizen and observer intake open | 2027-08-13 | T+3: M13 and web form live |
| IAB and accredited observer intake | 2027-08-15 | T+5: credentialed observer portal |
| First triage batch | 2027-08-20 | T+10: analyst console active |
| Public tracker published | 2027-08-17 | T+7: first public view |
| HIGH-severity departmental responses | 2027-09-10 | T+31 |
| IEBC Chair sign-off | 2027-10-10 | T+61 |
| Parliament referral | 2027-11-08 | T+90 |
| First long-drip 6-month review | 2028-02-10 | T+184 |
| Next-cycle handoff | 2031-01-01 | Start of 2032 cycle planning |

## 14. Dependencies

- M13 Public Verifier App — citizen intake channel
- M11 Public Portal — public tracker
- M15 IAB Dashboard — IAB submission
- M28 AI Monitoring Stack — deduplication clustering
- P05 Merkle Chain — sign-off and referral anchoring
- P06 HSM — Chair signing
- Parliament eCitizen portal MoU — external

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Low response rate from IEBC departments | Medium | High | Mandatory response deadline built into platform; IEBC Chair receives non-response alert at T+25 for HIGH findings |
| Long-drip scheduler silently fails | Low | High | Dead-letter queue on scheduled job; daily health-check alert; tested in integration suite via clock advance |
| Parliament referral package ignored | Medium | Medium | Platform tracks Parliament response; IEBC Chair publishes non-response status on public tracker after 90 days |
| Citizen submissions include PII about third parties | Medium | Medium | Intake form guidance; automated PII scan on free-text fields (redact before storage); ODPC DPIA covers residual risk |
| Recommendation set too large to action before next cycle | Low | Medium | Severity filter: HIGH findings are binding; MEDIUM deferred to planning register; LOW advisory only |

## 16. Cross-References

- `M11-public-portal.md` — public recommendation tracker
- `M13-public-verifier-app.md` — citizen intake
- `M15-iab-dashboard.md` — IAB submission and metrics
- `M22-campaign-finance-transparency.md` — finance findings input
- `M23-grievance-channels.md` — grievance findings input
- `M26-anomaly-fraud-detection.md` — security findings input
- `M28-ai-monitoring-stack.md` — deduplication clustering

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XXI.C.20 — long-drip attacks | FR-M41-009; NFR-M41-008; Long-Drip Register |
| §XXI — post-election review cycle | FR-M41-001 through FR-M41-013 |
| §XXI.C — systemic threat mitigation | FR-M41-009, FR-M41-010 |
| IEBC Act §7(1)(j) — annual reporting | Compliance Mapping; FR-M41-007 |

## 18. Decommissioning

**Trigger:** End of the 3-year IEMS framework (approximately 2030-08), or earlier if IEBC terminates the contract.

**Steps:**

1. **Data disposition** — all personal data purged or transferred to M69 (Long-Term Archive) per the retention schedule in DATA-MODEL.md §11 and DPA 2019 s.41. Disposal certificate signed by Datacraft DPO and IEBC DPO.
2. **Service shutdown** — Kubernetes deployments scaled to zero; PersistentVolumeClaims retained for 30 days then deleted after confirmation of M69 transfer.
3. **Credential revocation** — service account certificates, API keys, and HSM key handles revoked via P06 key-destruction ceremony within 48 hours of shutdown.
4. **Source-code escrow** — final tagged release deposited to C04 escrow within 7 days of shutdown.
5. **Runbook handover** — operational runbook and post-mortem notes delivered to IEBC ICT within 14 days.

**Owner:** SRE Lead. **Sign-off:** IEBC ICT Director + Datacraft Programme Director.

> **RESOLUTION (F-23, 2026-08-21):** Decommissioning stub added per audit finding. Owner: SRE Lead. Target: FAT-07 gate.
