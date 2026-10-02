# M58 — Parliamentary and Judiciary Reporting

**Module ID:** M58
**Name:** Parliamentary and Judiciary Reporting Platform
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in the Commission per GCC 15.4

**Strategy traceback:** Part II §PM-11, Part IX §IX.2.H, Part XII §XII.4, Part XIX §XIX.5, Part XXI.A.17

---

## 1. Purpose

the Commission Act 2011 §56 requires the Commission to submit an annual report to the National Assembly and the Senate, and a post-election report within six months of each general election. Beyond the statutory reports, the Commission appears before parliamentary committees (JLAC, relevant select committees) and must prepare testimony materials. On the Judiciary side, the Commission must maintain a structured handoff interface with the Chief Registrar and, for election-petition technology submissions, the pre-registered Supreme Court IT Registrar (upstream of M40 Election Court IT). This module provides: a statutory-report generation and tracking platform; a parliamentary committee testimony-preparation workspace; a formal Judiciary handoff registry; and a coordination channel for the SC IT Registrar pre-registration protocol. It ensures that every statutory deadline is tracked, every document submission is HSM-timestamped, and the evidence chain is court-ready.

Distinct from M40: M40 handles active election-petition IT interfaces during litigation. M58 handles the upstream pre-registration, the statutory parliamentary reporting cycle, and formal Judiciary handoffs that are not petition-driven.

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| the Commission Legal Counsel | Primary author of statutory reports | Full RW |
| the Commission CEO | Signatory on statutory reports | Approval and digital signature |
| the Commission Chair / Commissioner | Chair sign-off; parliamentary appearance | Read-all; sign statutory reports |
| Parliamentary Affairs Officer | Committee testimony coordination | RW on testimony workspace |
| Judiciary Liaison Officer | Chief Registrar handoff tracking | RW on Judiciary workspace |
| Datacraft Compliance Lead | Deadline tracking; format validation | RW on deadline registry |
| M40 IT Liaison | SC IT Registrar coordination | Read SC pre-registration records |
| IAB Secretariat | Cross-reference to IAB reports | Read statutory report drafts |

---

## 3. Functional Requirements

**FR-M58-01 — Statutory report registry:** Track all reports required by statute: the Commission Act §56 annual report (due within 3 months of financial year-end), post-election report (due within 6 months of polling day), quarterly parliamentary update (if required by committee). Each entry: statutory basis, due date, drafter, current status, submission confirmation. Overdue items escalate to the Commission CEO.

**FR-M58-02 — Report generation workspace:** Collaborative drafting environment for statutory reports. Structured sections with statutory mapping (which section satisfies which the Commission Act obligation). Version control with diff view. Review workflow: drafter → Legal Counsel → CEO → Chair sign-off. Each sign-off is captured as a digital signature event logged in P05.

**FR-M58-03 — Parliamentary committee testimony preparation:** Per committee appearance, a testimony-preparation workspace: committee name, date, subject matter, lead the Commission witness, briefing pack upload, Q&A anticipation tracker (question → draft answer → approved answer), and a post-appearance minutes upload. Briefing packs classified as internal; approved answers marked for potential public release.

**FR-M58-04 — Submission tracking:** Each report submitted to Parliament or Judiciary is tracked: submission method (physical, email, secure portal), submitted by, submitted at (timestamp), acknowledgement receipt. HSM-signed PDF of each submission stored permanently. Acknowledgement SLA: follow up if no acknowledgement within 5 business days.

**FR-M58-05 — Judiciary Chief Registrar handoff:** Registry of formal communications with the Judiciary Chief Registrar: election-period schedule, technology-system documentation submission, court-order compliance records. Each handoff: document type, transmission date, Chief Registrar reference number, acknowledgement status.

**FR-M58-06 — SC IT Registrar pre-registration:** Pre-election protocol (Strategy Part XII.4): coordinate with the Supreme Court IT Registrar for pre-registration of KIEMS cryptographic parameters, system documentation, and evidentiary-bundle structure before polling day. M58 tracks the pre-registration checklist: items submitted, SC IT Registrar acknowledgements, hash verification confirmations. Feeds upstream to M40 for active-petition use.

**FR-M58-07 — Statutory deadline calendar:** Integrated calendar of all parliamentary and judicial reporting deadlines, auto-calculated from election date (T-0 = 2027-08-10). Alert cascade: T-60, T-30, T-14, T-7 from each deadline. Alerts to the Commission CEO and Legal Counsel.

**FR-M58-08 — Document integrity:** Every submitted document generated as a signed PDF (P06 HSM). Document hash appended to P05 Merkle chain. Any document required for petition evidence retrievable from M40 via hash reference.

---

## 4. Non-Functional Requirements

**NFR-M58-01 — Performance:** Workspace p99 < 1.5 s; report PDF generation < 30 s for documents up to 200 pages.

**NFR-M58-02 — Reliability:** 99.5% uptime. During post-election petition window (T+7 to T+180), availability upgraded to 99.9%.

**NFR-M58-03 — Security:** MFA (TOTP + hardware key for Chair). Report drafts classified as internal until Chair signs. SC IT Registrar pre-registration records accessible only to the Commission Legal Counsel, CEO, and M40 liaison.

**NFR-M58-04 — WCAG 2.2 AA:** Internal staff-facing screens only (no public-facing UI beyond published statutory reports).

**NFR-M58-05 — DPA 2019 lawful basis:** Statutory reports processed under s.30(1)(b) (legal obligation). Testimony preparation may include personal data about candidates or voters only in aggregated or anonymised form.

**NFR-M58-06 — Audit:** Every action (draft save, version bump, sign-off, submission) logged in P05 with actor, timestamp, and document hash.

**NFR-M58-07 — SLA:** Statutory report submission before statutory deadline: 100%. SC IT Registrar pre-registration complete by T-14: 100%.

---

- **NFR-58-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-58` (replace 58 with the module number); alert ruleset ID `iems-58-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the Commission Legal Counsel, I view all upcoming statutory reporting deadlines with days-remaining and current draft status, so nothing falls through the gaps.
2. As the Commission Chair, I digitally sign the post-election annual report and the system generates a HSM-timestamped signed PDF submitted to the National Assembly.
3. As the Parliamentary Affairs Officer, I build a testimony briefing pack for a JLAC appearance, tracking each anticipated question and its approved answer.
4. As the Judiciary Liaison Officer, I log a formal document handoff to the Chief Registrar with submission date and track acknowledgement, so the record is on file.
5. As the M40 IT Liaison, I view the SC IT Registrar pre-registration checklist and confirm all 15 items are submitted before T-14.
6. As the Commission CEO, I approve the post-election report draft and receive confirmation that it has been submitted with HSM timestamp to Parliament.
7. As the Datacraft Compliance Lead, I run the statutory-deadline status report and confirm zero overdue items at T-30 before the election.
8. As the IAB Secretariat, I read the draft annual report to cross-check statistics against the IAB quarterly report findings.
9. As the Commission Legal Counsel, I generate a court-ready document bundle for the Chief Registrar including hash proofs for all submitted technology documentation.
10. As the Commission Chair, I view the full history of all parliamentary appearances and reports submitted in the current election cycle.
11. As the Parliamentary Affairs Officer, I upload committee-appearance minutes and link them to the testimony preparation workspace for the same appearance.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime, date
from typing import Literal

class StatutoryReport(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str()
    report_type: Literal["annual_report","post_election_report","quarterly_update","special_report"]
    statutory_basis: str     # e.g. "the Commission Act 2011 s.56"
    period_start: date
    period_end: date
    due_date: date
    status: Literal["not_started","drafting","under_review","signed","submitted","acknowledged"]
    drafter_id: str
    current_version: int
    signed_pdf_hash: str | None
    submitted_at: datetime | None
    submitted_to: str | None  # "National Assembly", "Senate", etc.
    acknowledgement_ref: str | None
    acknowledgement_at: datetime | None

class TestimonyWorkspace(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    committee_name: str
    appearance_date: date
    subject_matter: str
    lead_witness_id: str
    briefing_pack_hash: str | None
    minutes_hash: str | None
    status: Literal["upcoming","completed","cancelled"]
    questions: list[dict]   # {"question": str, "draft_answer": str, "approved": bool}

class JudiciaryHandoff(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    recipient: Literal["chief_registrar","sc_it_registrar","high_court_registrar"]
    document_type: str
    document_hash: str
    transmitted_at: datetime
    transmitted_by: str
    registrar_ref: str | None
    acknowledgement_at: datetime | None

class SCPreRegistrationItem(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    item_name: str
    description: str
    document_hash: str | None
    submitted_at: datetime | None
    sc_acknowledgement_at: datetime | None
    sc_hash_verification_confirmed: bool
    status: Literal["pending","submitted","acknowledged","verified"]
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/reports` | COMMISSION_LEGAL JWT | 60/min | List all statutory reports |
| POST | `/api/v1/reports` | COMMISSION_LEGAL JWT | 10/hr | Create new report entry |
| PATCH | `/api/v1/reports/{id}` | COMMISSION_LEGAL JWT | 30/hr | Update draft, advance status |
| POST | `/api/v1/reports/{id}/sign` | COMMISSION_CHAIR HSM | 5/hr | HSM sign and finalize |
| POST | `/api/v1/reports/{id}/submit` | COMMISSION_CEO JWT | 5/hr | Record submission to Parliament/Judiciary |
| GET | `/api/v1/reports/{id}/deadline-status` | the Commission JWT | 60/min | Days remaining, alert status |
| GET | `/api/v1/testimony` | the Commission JWT | 60/min | List committee appearances |
| POST | `/api/v1/testimony` | COMMISSION_PARL JWT | 10/hr | Create testimony workspace |
| PATCH | `/api/v1/testimony/{id}/questions/{qid}` | COMMISSION_PARL JWT | 30/hr | Update approved Q&A |
| POST | `/api/v1/judiciary/handoffs` | COMMISSION_LEGAL JWT | 10/hr | Log Judiciary handoff |
| GET | `/api/v1/judiciary/sc-preregistration` | COMMISSION_LEGAL or M40 JWT | 30/min | SC IT Registrar checklist |
| PATCH | `/api/v1/judiciary/sc-preregistration/{id}` | COMMISSION_LEGAL JWT | 10/hr | Update pre-registration item |
| GET | `/api/v1/deadlines/calendar` | the Commission JWT | 60/min | Full deadline calendar |

---

## 8. Integrations

- **M40 Election Court IT** — SC IT Registrar pre-registration data shared upstream
- **M15 IAB Dashboard** — IAB reads statutory report drafts for cross-reference
- **P05 Audit Log** — all actions and document hashes logged
- **P06 HSM** — Chair digital signature on statutory reports; document timestamping
- **Parliament (external)** — submission confirmation tracking; no API integration assumed; manual acknowledgement logged
- **Judiciary Chief Registrar (external)** — formal handoff logging; manual acknowledgement

---

## 9. UI/UX Key Screens

1. **Deadline Dashboard** — traffic-light calendar of all statutory and judicial deadlines; days-remaining counters; status badges.
2. **Report Workspace** — structured section editor with statutory-mapping sidebar; version history; reviewer comment threads; sign-off workflow panel.
3. **Testimony Builder** — committee appearance summary card; briefing pack upload; Q&A tracker with approved/pending status per question.
4. **Judiciary Registry** — list of all handoff documents with transmission and acknowledgement dates; link to document hashes.
5. **SC Pre-Registration Checklist** — 15-item checklist with status per item; hash verification column; M40 liaison read view.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| Annual report to Parliament | the Commission Act 2011 s.56(1) |
| Post-election report | the Commission Act 2011 s.56(2) |
| Parliamentary committee appearance | National Assembly (Powers and Privileges) Act Cap 6 |
| Judiciary document submission | Supreme Court Act 2011; Supreme Court Rules 2012 r.10 |
| Pre-election SC IT Registrar registration | Strategy Part XII.4; Raila v the Commission [2017] eKLR para 276 |
| Document integrity and audit trail | Evidence Act Cap 80; Public Archives Act Cap 19 |
| DPA 2019 processing lawful basis | DPA 2019 s.30(1)(b) — legal obligation |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| Statutory reports submitted on time | 100% before statutory deadline | the Commission CEO |
| SC IT Registrar pre-registration complete | By T-14 (100%) | the Commission Legal Counsel |
| Document acknowledgement follow-up | 100% within 5 business days of submission | Parliamentary Affairs Officer |
| Deadline alert delivery | 100% at T-60, T-30, T-14, T-7 | Datacraft SRE |
| Report signing event in P05 audit log | 100% of signed documents | Datacraft Compliance Lead |

---

## 12. Test Strategy

- **Unit:** Deadline calculator from election date; sign-off state machine; SC pre-registration checklist completion logic.
- **Integration:** Report PDF signing via P06 HSM; hash anchoring in P05 Merkle chain; M40 pre-registration data feed.
- **E2E:** Full annual report cycle from creation through parliamentary submission and acknowledgement.
- **Access control:** Only the Commission Chair can execute HSM sign; M40 liaison cannot modify pre-registration items.
- **Deadline regression:** Insert a statutory deadline and confirm alert fires at all four milestones.

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| Statutory calendar seeded | T-365 (2026-10-10) | All deadlines entered; alerts configured |
| Annual report 2026 drafted in system | T-270 (2027-01-13) | First statutory report through full workflow |
| Judiciary handoff protocol agreed | T-180 (2027-02-10) | Chief Registrar MoU signed |
| SC IT Registrar pre-registration begins | T-60 (2027-06-11) | First items submitted |
| Pre-registration complete | T-14 (2027-07-27) | All 15 items acknowledged by SC IT Registrar |
| Post-election report drafted | T+90 (2027-11-08) | Report submitted within 6 months of T-0 |

---

## 14. Dependencies

- **M40 Election Court IT** — pre-registration data upstream
- **M15 IAB Dashboard** — statutory report cross-reference
- **P05, P06** — audit log and HSM signing
- Parliament of Kenya — submission acknowledgement (manual)
- Judiciary Chief Registrar office — formal document acceptance

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Parliamentary committee cancels or reschedules with < 7 days notice | Testimony workspace pre-loads standard the Commission briefing pack; the Commission Chair has read-ready summary always current |
| R2 | SC IT Registrar office changes contact point or protocol | Pre-register via Judiciary ICT Director as backup; cross-check at T-90, T-60, T-30; all contact details in M40 registry |
| R3 | Statutory report deadline missed due to Chair travel | Chair HSM token delegable to CEO with written authority; M58 tracks delegation events in audit log |
| R4 | Parliament requests unplanned special report on short notice | Report generation workspace supports ad-hoc report creation; standard statutory-mapping templates available |
| R5 | Document hash mismatch during petition (report submitted vs. hash in M40) | Every submission event logs both the P05 Merkle path and the submission timestamp; M40 can independently verify via P05 |

---

## 16. Cross-References

- **M40** — Election Court IT: SC IT Registrar pre-registration feeds M40 active petition interface
- **M15** — IAB Dashboard: reads statutory report publication status
- **M59** — Auditor-General Interface: audit reports may trigger special parliamentary reporting
- **M22** — Campaign Finance Transparency: post-election campaign finance report included in post-election statutory report

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| Statutory reporting to Parliament | FR-M58-01, FR-M58-02, FR-M58-07 |
| Parliamentary committee testimony readiness | FR-M58-03, FR-M58-04 |
| Judiciary Chief Registrar handoff | FR-M58-05 |
| SC IT Registrar pre-election registration | FR-M58-06 |
| Document integrity for legal proceedings | FR-M58-08, NFR-M58-06 |

## 18. Decommissioning

**Trigger:** End of the 3-year IEMS framework (approximately 2030-08), or earlier if the Commission terminates the contract.

**Steps:**

1. **Data disposition** — all personal data purged or transferred to M69 (Long-Term Archive) per the retention schedule in DATA-MODEL.md §11 and DPA 2019 s.41. Disposal certificate signed by Datacraft DPO and the Commission DPO.
2. **Service shutdown** — Kubernetes deployments scaled to zero; PersistentVolumeClaims retained for 30 days then deleted after confirmation of M69 transfer.
3. **Credential revocation** — service account certificates, API keys, and HSM key handles revoked via P06 key-destruction ceremony within 48 hours of shutdown.
4. **Source-code escrow** — final tagged release deposited to C04 escrow within 7 days of shutdown.
5. **Runbook handover** — operational runbook and post-mortem notes delivered to the Commission ICT within 14 days.

**Owner:** SRE Lead. **Sign-off:** the Commission ICT Director + Datacraft Programme Director.

> **RESOLUTION (F-23, 2026-08-21):** Decommissioning stub added per audit finding. Owner: SRE Lead. Target: FAT-07 gate.
