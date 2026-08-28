# M36 — Voter Deregistration Appeal

**Module ID:** M36
**Name:** Voter Deregistration and Statutory Appeal Workflow
**Runs on:** Central servers (Kenyan cloud) + IEBC back-office
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Provide the statutory workflow for removing a voter from the register and for adjudicating that voter's (or their representative's) appeal against removal. Enforce ODPC-compliant data-minimisation on deregistered records. Publish a quarterly deregistration report on the public portal (M11). Distinct from M35 (which proposes removals based on agency feeds); this module manages the case-officer triage, appeal intake, adjudicator sign-off, and gazette publication.

Strategy reference: §XXI.B.15. Constitution Art 38 (right to vote), DPA 2019 §26 (right of erasure and objection).

## 2. User Personas

- **IEBC Case Officer (Deregistration)** — triages proposed removals from M35; issues notice of intent to deregister
- **IEBC Adjudicator** — hears appeals; issues written decisions; independent of case officer
- **Voter (appellant)** — receives deregistration notice; files appeal via app, web, short-code, or in-person
- **Legal Representative** — files appeal on behalf of appellant; uploads authorisation
- **IEBC Commissioner** — countersigns gazette of final deregistrations
- **ODPC Compliance Lead** — audits lawful basis and data-minimisation post-deregistration
- **IAB Observer** — views aggregate quarterly deregistration metrics

## 3. Functional Requirements

- **FR-M36-001** Deregistration notice: for every M35-approved proposed removal, generate a notice of intent to deregister (PDF) addressed to the voter's registered contact; dispatch via SMS link + postal address + M13 in-app notification. Notice period: 30 days per Elections Act §8A.
- **FR-M36-002** Appeal intake: voter or representative files appeal within 30-day notice period via: (a) M13 Public Verifier App, (b) web form, (c) *IEBC# USSD, (d) in-person IAB office. Appeal creates a case record with appeal_id (uuid7), intake channel, appellant identity, grounds, and supporting documents.
- **FR-M36-003** Case-officer triage: case officer reviews appeal grounds; categorises as (IDENTITY_ERROR / ALIVE_AND_PRESENT / DATA_MATCH_ERROR / FAMILY_DISPUTE); requests additional evidence if needed; recommends UPHOLD_REMOVAL or OVERTURN to adjudicator.
- **FR-M36-004** Adjudicator hearing: adjudicator views full case file (case officer recommendation, evidence, M35 source data summary — not raw CRS/NRB data); issues written decision within 14 days of hearing; decision is final for administrative purposes; voter retains right to court petition.
- **FR-M36-005** OVERTURN path: if adjudicator overturns, M35 event is reversed; ABIS soft-hold lifted; voter record restored with REINSTATED status; voter notified within 24 hours.
- **FR-M36-006** UPHOLD path: record moved to DEREGISTERED status; ABIS template soft-deleted (recoverable for 7 years audit period); contact data purged per DPA 2019 §26; voter notified with right-to-petition advisory.
- **FR-M36-007** No-appeal expiry: if no appeal filed within 30-day notice period, case officer confirms; record moves to DEREGISTERED after IEBC Commissioner countersign.
- **FR-M36-008** ODPC compliance: before final deregistration, system checks DPIA register for the processing basis; logs the data-minimisation action (purge of contact, address, photo) with officer_id, timestamp, and DPA basis code; generates compliance receipt stored for 7 years.
- **FR-M36-009** Quarterly deregistration report: aggregated report (total proposed, total noticed, total appealed, total overturned, total confirmed deregistered) per county; signed by IEBC Commissioner; published on M11 public portal.
- **FR-M36-010** Report anchoring: quarterly report PDF hash anchored in P05 Merkle chain at publication.
- **FR-M36-011** Multi-language: notices in EN + SW + county-detected Kenyan language; appeal intake supports EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya.
- **FR-M36-012** Deadline tracking: system enforces 30-day notice period, 14-day adjudication SLA; alerts supervisor when either deadline is at risk.

## 4. Non-Functional Requirements

- **NFR-M36-001** Notice dispatch: <= 1 hour from M35 approval event to voter SMS/notification.
- **NFR-M36-002** Appeal intake: p99 <= 500 ms; available >= 99.5%.
- **NFR-M36-003** Adjudicator hearing SLA: <= 14 days from appeal receipt to decision.
- **NFR-M36-004** WCAG 2.2 AA: appeal intake web form; M13 app integration; status tracker.
- **NFR-M36-005** DPA 2019 §26: right-of-erasure action (contact/address/photo purge) completed within 30 days of confirmed deregistration; logged to ODPC-accessible audit trail.
- **NFR-M36-006** Data segregation: raw CRS/NRB/KRA source documents never stored in the appeal case file; case officer sees only the IEBC-generated summary.
- **NFR-M36-007** Audit trail: every status transition, officer action, adjudicator decision, and data-minimisation event stored immutably for 7 years.
- **NFR-M36-008** Scalability: handle up to 10,000 concurrent deregistration cases during a post-census catch-up period.

- **NFR-36-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-36` (replace 36 with the module number); alert ruleset ID `iems-36-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a voter, I receive an SMS with a link to a notice of intent to deregister me (due to a CRS death record); I open the link and see the grounds and appeal deadline in Swahili.
2. As a voter, I file an appeal via *IEBC# USSD stating I am alive and present; I receive a case reference by SMS within 5 minutes.
3. As an IEBC Case Officer, I review an appeal categorised as IDENTITY_ERROR (wrong ID number in CRS batch); I recommend OVERTURN and upload the corrected ID scan.
4. As an IEBC Adjudicator, I see the case officer's recommendation and all uploaded evidence; I issue an OVERTURN decision in writing; the system automatically reinstates the voter record.
5. As a legal representative, I upload a power of attorney and file an appeal on behalf of an elderly appellant who cannot use digital channels; the system accepts the authorisation and creates the case.
6. As the IEBC Commissioner, I countersign the quarterly deregistration report; the signed PDF is published on the public portal and anchored in the Merkle chain.
7. As an IAB Observer, I view this quarter's deregistration metrics broken down by county and grounds category on the IAB dashboard.
8. As an ODPC Compliance Lead, I pull the data-minimisation log to confirm that all deregistered voters' contact data was purged within 30 days of confirmation.
9. As a voter whose appeal was upheld (removal confirmed), I receive a notification with the adjudicator's written decision and a reference to my right to petition the High Court under Constitution Art 38.
10. As an IEBC Supervisor, I see a dashboard of cases with upcoming 30-day notice expiry and 14-day adjudication SLA breaches highlighted in red.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime

class DeregistrationCase(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    voter_id: str
    m35_event_id: str               # source event from M35
    status: Literal[
        "NOTICED","APPEAL_FILED","UNDER_REVIEW","ADJUDICATION",
        "OVERTURNED","CONFIRMED","DEREGISTERED","EXPIRED_NO_APPEAL"
    ]
    notice_dispatched_at: datetime | None
    notice_deadline: datetime       # notice_dispatched_at + 30 days
    appeal_id: str | None
    officer_id: str | None
    officer_recommendation: Literal["UPHOLD_REMOVAL","OVERTURN"] | None
    adjudicator_id: str | None
    adjudicator_decision: Literal["UPHOLD_REMOVAL","OVERTURN"] | None
    adjudicator_decision_at: datetime | None
    adjudicator_notes: str | None
    data_minimisation_at: datetime | None
    data_minimisation_officer: str | None
    created_at: datetime

class AppealRecord(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    case_id: str
    intake_channel: Literal["VERIFIER_APP","WEB","USSD","IN_PERSON"]
    appellant_id_hash: str          # SHA-256 of National ID
    representative_auth_file_id: str | None
    grounds: Literal[
        "IDENTITY_ERROR","ALIVE_AND_PRESENT","DATA_MATCH_ERROR","FAMILY_DISPUTE","OTHER"
    ]
    grounds_detail: str
    supporting_doc_ids: list[str]
    language: str                   # BCP-47
    filed_at: datetime
    hearing_date: datetime | None

class QuarterlyDeregistrationReport(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    period_start: str               # YYYY-MM-DD
    period_end: str
    county_breakdown: dict[str, dict]  # county_code -> {proposed, noticed, appealed, overturned, confirmed}
    total_proposed: int
    total_noticed: int
    total_appealed: int
    total_overturned: int
    total_confirmed: int
    report_pdf_hash: str
    merkle_inclusion_proof: str | None
    commissioner_signed_at: datetime | None
    published_at: datetime | None
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/deregistration/cases | M35_INTERNAL service | 500/hr | Create deregistration case from M35 approval |
| GET | /v1/deregistration/cases | IEBC_OFFICER JWT | 200/hr | List cases in triage queue |
| GET | /v1/deregistration/cases/{id} | IEBC_OFFICER JWT | 200/hr | Get case detail |
| PATCH | /v1/deregistration/cases/{id} | IEBC_OFFICER JWT | 100/hr | Update triage recommendation |
| POST | /v1/deregistration/cases/{id}/appeals | Public + case reference | 50/hr per IP | File an appeal |
| GET | /v1/deregistration/cases/{id}/appeals/{aid} | IEBC_ADJUDICATOR JWT | 100/hr | Get appeal record |
| POST | /v1/deregistration/cases/{id}/decision | IEBC_ADJUDICATOR JWT | 20/hr | Record adjudicator decision |
| POST | /v1/deregistration/cases/{id}/minimise | IEBC_OFFICER JWT | 50/hr | Execute data-minimisation action |
| GET | /v1/deregistration/reports | Public | 200/min | List quarterly deregistration reports |
| GET | /v1/deregistration/reports/{id} | Public | 200/min | Download report PDF + Merkle proof |
| POST | /v1/deregistration/reports/{id}/sign | IEBC_COMMISSIONER HSM | 2/quarter | Commissioner sign and publish report |

## 8. Integration Points

- **M35 CVR** — source of proposed-removal events
- **M02 BVR Back-End** — voter record reinstatement or final-status update
- **M03 ABIS** — soft-delete or lift of biometric soft-hold
- **M13 Public Verifier App** — appellant self-service portal
- **M11 Public Portal** — quarterly report publication
- **M15 IAB Dashboard** — aggregate metrics
- **M40 Election Court IT Interface** — adjudicator decisions available to petitioners
- **P05 Merkle Chain** — quarterly report anchoring
- **P06 HSM** — Commissioner countersigning
- **Africa's Talking / Safaricom** — *IEBC# USSD appeal intake
- **ODPC** — DPIA register lookup and compliance receipt submission

## 9. UI/UX Key Screens

1. **Voter Notice Page (WCAG 2.2 AA)** — displays grounds, evidence summary (non-technical), 30-day deadline countdown, appeal button; available in EN + SW + county language.
2. **Appeal Intake Form** — grounds selector; free-text detail; document upload (PDF, image); representative auth upload; language selector; accessible keyboard navigation.
3. **Case Officer Console** — case list with status badges and SLA countdown; case detail: appellant grounds, officer note field, recommendation radio (UPHOLD/OVERTURN); evidence upload.
4. **Adjudicator Workspace** — read-only view of case officer recommendation + evidence + M35 source summary; decision field (UPHOLD/OVERTURN) with mandatory written rationale; submit with adjudicator credential.
5. **Commissioner Report Sign-Off** — quarter summary table; total figures; preview PDF; HSM sign button; confirmation modal.
6. **ODPC Compliance Log** — paginated list of data-minimisation actions: voter_id (masked), action type, officer, timestamp, DPA basis code; exportable CSV.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Right to vote (appeal basis) | Constitution Art 38 |
| Register correction and deregistration | Elections Act 2011 §8A |
| 30-day notice period | Elections Act 2011 §8A(2) |
| Right to object to data processing | DPA 2019 §26 |
| Data minimisation and erasure | DPA 2019 §26; IEBC DPIA |
| Adjudicator independence | IEBC Act 2011 §9 (commissioners as adjudicators) |
| Court petition right post-appeal | Constitution Art 38; Elections Act §75 |
| Gazette of deregistrations | Elections Act 2011 §15 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Notice dispatch within SLA | 100% within 1 hour of M35 approval | Dispatch timestamp | Datacraft SRE |
| Appeal acknowledgement | <= 5 minutes from submission | Case-creation timestamp | IEBC Ops |
| Adjudication SLA compliance | >= 95% within 14 days | SLA tracking log | IEBC Commissioner |
| Data-minimisation completion | 100% within 30 days of confirmed deregistration | ODPC audit log | ODPC DPO |
| Quarterly report published on schedule | 100% within T+7 of quarter-end | Publication timestamp | IEBC Commissioner |
| Report Merkle anchor | 100% | Merkle reconciliation | Datacraft Technical |
| Zero PII exposed in public report | 100% | Automated PII scan on report | Datacraft CISO |

## 12. Test Strategy

- **Unit:** notice-deadline calculator; appeal-grounds classifier; data-minimisation action trigger.
- **Integration:** M35 event to case creation; appeal intake via USSD webhook; M03 ABIS soft-hold lift on OVERTURN; M11 report publication.
- **E2E:** simulate CRS-based removal; voter appeals via web; case officer recommends OVERTURN; adjudicator confirms; verify voter record reinstated and ABIS hold lifted.
- **Security:** attempt to access another voter's appeal case; attempt to access raw CRS source data via appeal API; verify DPA minimisation action irreversibility.
- **Accessibility:** WCAG 2.2 AA on notice page, appeal form, status tracker (axe-core + manual + screen-reader).
- **Load:** 5,000 concurrent appeal submissions during a large post-census deregistration batch.
- **Chaos:** M35 event queue backlog — verify case creation does not block notice dispatch.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Module integration with M35 | 2027-01-15 | End-to-end case creation from M35 events |
| Officer and adjudicator UAT | 2027-02-15 | 10 officers + 3 adjudicators with synthetic cases |
| *IEBC# USSD appeal channel live | 2027-03-01 | Integrated with grievance short-code |
| ODPC DPIA registered | 2027-03-15 | Compliance pre-condition |
| First live quarterly report | 2027-04-07 | Q1 2027 deregistration report |
| Register freeze | 2027-07-01 | T-40: no new deregistrations processed until T+7 |
| Post-election analytics | 2027-09-10 | T+31: full-cycle deregistration report |

## 14. Dependencies

- M35 CVR — proposed-removal events
- M02 BVR Back-End — voter record management
- M03 ABIS — biometric hold management
- M13 Public Verifier App — voter self-service
- M11 Public Portal — public report
- M40 Election Court IT Interface — adjudicator decisions for petitioners
- P05 Merkle Chain, P06 HSM — report anchoring and signing
- ODPC — DPIA registration

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| High appeal volume overwhelming adjudicators | Medium | High | Scalable adjudicator pool (IEBC can assign additional commissioners); case officer pre-filters weak appeals |
| Voter unreachable at registered contact | High | Medium | Multi-channel notice (SMS + postal + M13 push); public gazette as constructive notice |
| Adjudicator decision challenged in court | Low | Medium | Written decision required; evidence trail auditable; Merkle-anchored record admissible |
| DPA minimisation action executed prematurely | Low | High | 7-year audit-hold flag prevents irreversible purge during appeal window; officer confirmation required |
| ODPC DPIA registration delayed | Medium | Medium | Datacraft prepares draft DPIA at contract start; IEBC submits within 30 days of award |

## 16. Cross-References

- `M35-cvr-continuous-voter-register.md` — proposed-removal source
- `M02-bvr-backend.md` — voter record reinstatement
- `M03-abis.md` — biometric hold management
- `M11-public-portal.md` — public report publication
- `M13-public-verifier-app.md` — voter appeal self-service
- `M40-election-court-it-interface.md` — adjudicator decisions for court use

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XXI.B.15 — deregistration workflow | FR-M36-001 through FR-M36-009 |
| §XXI.B.15 — ODPC compliance | NFR-M36-005, FR-M36-008 |
| Constitution Art 38 — right to vote | FR-M36-002, FR-M36-004, Compliance Mapping |
| DPA 2019 §26 — data minimisation | FR-M36-006, FR-M36-008, NFR-M36-005 |

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
