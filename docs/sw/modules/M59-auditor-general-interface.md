# M59 — Auditor-General Interface

**Module ID:** M59
**Name:** Auditor-General Interface
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part II §PM-12, Part IX §IX.2.I, Part XII §XII.2, Part XIX §XIX.6, Part XXI.A.18

---

## 1. Purpose

The Office of the Auditor-General (OAG) audits IEBC annually under Constitution Art. 229 and the Public Audit Act 2015. During election years, OAG may conduct election-specific special audits covering procurement of KIEMS kits, kit-deployment costs, results-transmission costs, and biometric-registration expenditure. This module provides a controlled data-production interface between KIEMS and OAG: scope negotiation for each audit engagement, a structured evidence-production API that pulls audit-relevant records from across KIEMS modules, an IEBC-controlled response-tracking workflow, and integration with published OAG audit reports. Distinct from M40 (Election Court IT) and M15 (IAB Dashboard): OAG is a constitutional oversight body with a separate legal mandate, separate documentary requirements, and a distinct engagement lifecycle from IAB or court processes.

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| IEBC Internal Auditor | Primary OAG liaison | Full RW on audit workspaces |
| IEBC CFO | Scope approval; management response sign-off | RW on scope and responses |
| OAG Audit Manager | Leads OAG engagement | Read produced evidence; submit audit queries |
| OAG IT Auditor | Reviews KIEMS technology audit scope | Technical evidence access within agreed scope |
| IEBC CEO | Sign-off on management responses | Approval and digital signature |
| Datacraft Finance Lead | KIEMS cost data production | Provide data extract within scope |
| M15 IAB Secretariat | Cross-reference audit findings | Read published OAG reports only |

---

## 3. Functional Requirements

**FR-M59-01 — Audit engagement registry:** Each OAG audit engagement registered: type (annual, special-election, follow-up), financial year, engagement letter reference, OAG contact, IEBC liaison, scope statement. Lifecycle: notified, scope-agreed, evidence-production, management-response, report-published, follow-up.

**FR-M59-02 — Scope negotiation workflow:** For each engagement, IEBC Internal Auditor and OAG Audit Manager co-author and agree the audit scope document. Scope defines: KIEMS modules in scope, data categories, date ranges, access level (summary, transaction-level, raw log). Scope locked with dual sign-off (IEBC CFO + OAG Audit Manager) before evidence production begins. Scope changes require new dual sign-off.

**FR-M59-03 — Evidence production API:** Controlled pull from KIEMS modules within agreed scope. Evidence categories: financial transactions (from IEBC procurement system — external), kit-inventory events (from M20), biometric-registration counts (from M02/M03), results-transmission metadata (from M10), kit-procurement cost data (from IEBC ERP — external adapter). Each data extract: scope reference, extraction timestamp, record count, SHA-256 hash. Extracted packages signed by P06 HSM. OAG receives signed package via secure transfer channel.

**FR-M59-04 — Query management:** OAG submits written audit queries through the portal. Each query: OAG reference, query text, KIEMS modules implicated, response deadline. IEBC Internal Auditor assigns to module owner. Response drafted, reviewed by CFO or CEO, and submitted through portal with digital signature. Query history retained permanently.

**FR-M59-05 — Management response workflow:** OAG issues draft audit findings to IEBC. IEBC management response drafted collaboratively (Internal Auditor + CFO + CEO). Version-controlled. CEO signs final response. Response submitted through portal with HSM timestamp. Deadline: statutory 30 days from draft finding receipt (Public Audit Act 2015 s.32).

**FR-M59-06 — Published report integration:** When OAG publishes an audit report concerning IEBC, IEBC Internal Auditor registers the report in M59: OAG report reference, publication date, key findings, follow-up action items with owners and due dates. Action-item status tracked until closed. M15 IAB can read published report metadata (not IEBC internal audit workspaces).

**FR-M59-07 — Access-log compliance:** Every OAG access event (which data category, which record count, which extraction timestamp, which OAG user, under which scope reference) logged in P05. DPA 2019 s.26 purpose code attached to every log entry. IEBC Internal Auditor can generate a complete OAG-access report for any period.

**FR-M59-08 — Audit-findings follow-up tracker:** Post-audit, each OAG recommendation logged with: finding severity (material, significant, minor), IEBC response (accepted, partially accepted, rejected with reason), implementation owner, target date, completion evidence. Follow-up status shared with OAG at 6-month intervals.

---

## 4. Non-Functional Requirements

**NFR-M59-01 — Performance:** Audit workspace p99 < 2 s. Evidence extraction for up to 10M records completes within 30 minutes; extract status available via polling endpoint.

**NFR-M59-02 — Reliability:** 99.5% uptime. Evidence production must be available within 5 business days of scope sign-off.

**NFR-M59-03 — Security:** OAG users receive time-limited IEBC-issued credentials scoped exactly to the agreed audit scope. No OAG credential allows broader access than the locked scope document. All OAG sessions recorded in access log. Credentials expire at engagement close.

**NFR-M59-04 — Data minimisation:** Evidence production extracts only records within agreed scope. No voter biometric data in scope by default; any biometric audit requires separate ODPC notification and DPA 2019 s.30(1)(b) justification.

**NFR-M59-05 — DPA 2019 lawful basis:** Audit data processed under s.30(1)(b) (legal obligation — Public Audit Act 2015). DPIA filed with ODPC before any election-special audit involving personal data. Retention: 7 years post-audit.

**NFR-M59-06 — SLA:** Management response submitted within 30 days of draft finding receipt (Public Audit Act 2015 s.32). Evidence package delivered to OAG within 5 business days of scope sign-off.

**NFR-M59-07 — WCAG 2.2 AA:** Internal IEBC staff screens.

---

- **NFR-59-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-59` (replace 59 with the module number); alert ruleset ID `iems-59-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the IEBC Internal Auditor, I register a new OAG special-election audit engagement, enter the scope draft, and invite the OAG Audit Manager to co-author the scope document.
2. As the OAG Audit Manager, I propose a scope amendment adding kit-procurement data, and the system requires IEBC CFO sign-off before the amended scope takes effect.
3. As the IEBC CFO, I review and digitally sign the agreed audit scope, locking evidence production to that scope.
4. As the OAG IT Auditor, I request a data extract of all kit-inventory events for the election cycle; the system produces a signed package within 30 minutes and logs the extraction event.
5. As the IEBC Internal Auditor, I receive an OAG audit query about a discrepancy in transmission costs, assign it to the Datacraft Finance Lead, and track the response deadline.
6. As the IEBC CEO, I sign the management response to OAG draft findings, and the portal submits it with HSM timestamp to the OAG query thread.
7. As the IEBC Internal Auditor, I register the published OAG report and create five follow-up action items, each assigned to a module owner with a due date.
8. As the M15 IAB Secretariat, I view the list of published OAG reports and their key-finding summaries, so I can reference them in the IAB quarterly report.
9. As the IEBC DPO, I generate an OAG access report showing every data category accessed by OAG during the engagement, with DPA purpose codes.
10. As the Datacraft Finance Lead, I receive an evidence-production task for KIEMS cost data within the agreed scope and upload the signed extract within the 5-day SLA.
11. As the IEBC Internal Auditor, I close a follow-up action item after the implementation owner uploads completion evidence, and the system notifies the OAG Audit Manager.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime, date
from typing import Literal

class AuditEngagement(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str()
    audit_type: Literal["annual","special_election","follow_up"]
    financial_year: str      # e.g. "2026/27"
    oag_engagement_ref: str
    oag_contact: str
    iebc_liaison_id: str
    scope_summary: str
    scope_locked_at: datetime | None
    iebc_cfo_signed_at: datetime | None
    oag_manager_signed_at: datetime | None
    status: Literal["notified","scope_agreed","evidence_production","management_response","report_published","follow_up_open","closed"]
    created_at: datetime

class EvidenceExtract(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    engagement_id: str
    scope_ref: str
    data_category: str       # e.g. "kit_inventory_events", "rts_metadata", "brn_counts"
    source_module: str
    date_range_start: date
    date_range_end: date
    record_count: int
    extract_hash: str
    extracted_at: datetime
    delivered_to_oag_at: datetime | None
    oag_acknowledgement_at: datetime | None
    dpa_purpose_code: str

class AuditQuery(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    engagement_id: str
    oag_ref: str
    query_text: str
    modules_implicated: list[str]
    response_deadline: date
    assigned_to: str
    status: Literal["open","draft_response","submitted","closed"]
    response_text: str | None
    response_signed_by: str | None
    response_signed_at: datetime | None

class ManagementResponse(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    engagement_id: str
    draft_finding_received_at: datetime
    statutory_deadline: datetime
    response_pdf_hash: str | None
    ceo_signed_at: datetime | None
    submitted_at: datetime | None

class AuditFollowUpItem(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    engagement_id: str
    oag_report_ref: str
    finding_severity: Literal["material","significant","minor"]
    finding_description: str
    iebc_response: Literal["accepted","partially_accepted","rejected"]
    rejection_reason: str | None
    owner_id: str
    target_date: date
    completion_evidence_hash: str | None
    status: Literal["open","in_progress","completed","overdue"]
    closed_at: datetime | None
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/audit-engagements` | IEBC_AUDIT JWT | 60/min | List all engagements |
| POST | `/api/v1/audit-engagements` | IEBC_AUDIT JWT | 5/hr | Register new engagement |
| PATCH | `/api/v1/audit-engagements/{id}/scope` | IEBC_CFO or OAG JWT | 10/hr | Co-author scope |
| POST | `/api/v1/audit-engagements/{id}/scope/lock` | IEBC_CFO + OAG JWT | 3/hr | Dual sign-off to lock scope |
| POST | `/api/v1/audit-engagements/{id}/extracts` | IEBC_AUDIT JWT | 5/hr | Initiate evidence production |
| GET | `/api/v1/audit-engagements/{id}/extracts/{eid}/status` | IEBC or OAG JWT | 30/min | Extract status + hash |
| GET | `/api/v1/audit-engagements/{id}/queries` | IEBC or OAG JWT | 60/min | List audit queries |
| POST | `/api/v1/audit-engagements/{id}/queries` | OAG JWT | 10/hr | Submit audit query |
| POST | `/api/v1/audit-engagements/{id}/queries/{qid}/respond` | IEBC_CEO JWT | 5/hr | Submit signed response |
| GET | `/api/v1/audit-engagements/{id}/management-response` | IEBC JWT | 30/min | Management response status |
| POST | `/api/v1/audit-engagements/{id}/management-response/sign` | IEBC_CEO HSM | 3/hr | HSM sign and submit |
| GET | `/api/v1/audit-engagements/{id}/followup` | IEBC or M15 JWT | 30/min | Follow-up action items |
| PATCH | `/api/v1/audit-engagements/{id}/followup/{fid}` | IEBC_AUDIT JWT | 20/hr | Update action item |
| GET | `/api/v1/oag-access-log` | IEBC_DPO JWT | 20/min | OAG access audit report |

---

## 8. Integrations

- **M20 IMS** — kit-inventory event extract within agreed scope
- **M10 RTS Backend** — transmission metadata extract
- **M02/M03 BVR** — biometric registration counts (summary only by default)
- **M15 IAB Dashboard** — published OAG report metadata available to IAB
- **M58 Parliamentary Reporting** — OAG audit findings may trigger special parliamentary report
- **P05 Audit Log** — all access and evidence events logged
- **P06 HSM** — evidence package signing; CEO management-response signing
- **IEBC ERP** (external) — procurement and cost data via adapter (to be spec'd post-award)

---

## 9. UI/UX Key Screens

1. **Engagement Registry** — engagement list with status badges; days-to-statutory-deadline counter; action alerts for overdue items.
2. **Scope Editor** — co-authored document with OAG; section locking after sign-off; diff view for scope amendments.
3. **Evidence Production Console** — data category selector (constrained to scope); extract initiation; status polling; delivery confirmation and OAG acknowledgement tracking.
4. **Query Tracker** — kanban of OAG queries: open / draft response / submitted / closed; deadline countdown; signed-response workflow.
5. **Management Response Builder** — draft editor; version history; CEO sign-off panel; statutory deadline countdown.
6. **Follow-Up Dashboard** — action items by severity; owner and due date; completion evidence upload; OAG notification on closure.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| Auditor-General mandate | Constitution Art. 229 |
| Public Audit Act obligations | Public Audit Act 2015 s.10, s.32 |
| IEBC financial accountability | Public Finance Management Act 2012 s.68 |
| Management response deadline | Public Audit Act 2015 s.32 (30 days) |
| DPA 2019 lawful basis | DPA 2019 s.30(1)(b) — legal obligation |
| Evidence integrity | Evidence Act Cap 80; Digital Security Act 2023 |
| Retention of audit records | Public Archives and Documentation Service Act Cap 19 |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| Management response within 30-day statutory deadline | 100% | IEBC CEO |
| Evidence package delivered within 5 business days of scope lock | 100% | IEBC Internal Auditor |
| OAG access events logged with DPA purpose code | 100% | IEBC DPO |
| Follow-up action items closed by target date | >= 90% | IEBC Internal Auditor |
| Published OAG reports registered within 5 business days of publication | 100% | IEBC Internal Auditor |

---

## 12. Test Strategy

- **Unit:** Scope-lock dual sign-off enforcement; statutory deadline calculator; evidence-extract hash generation.
- **Integration:** Evidence pull from M20, M10, M02/M03 within scope constraints; hash anchoring in P05.
- **Access control:** OAG credentials scoped exactly to locked scope; IEBC CFO cannot access OAG-only views; M15 reads only published report metadata.
- **E2E:** Full special-election audit cycle: engagement registration, scope lock, evidence production, OAG query, management response, follow-up.
- **Security:** Attempt OAG credential access outside agreed scope; verify blocked at API gateway.
- **Load:** Simultaneous evidence extracts for 3 concurrent data categories within 30-minute SLA.

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| IEBC Internal Auditor onboarded | T-365 (2026-10-10) | System access and training |
| Annual audit 2025/26 engagement registered | T-300 (2026-12-14) | First engagement through full workflow |
| Special-election audit scope template ready | T-180 (2027-02-10) | Template agreed with OAG |
| Election-specific data adapters tested | T-90 (2027-05-10) | Evidence extracts from M20 + M10 tested |
| Post-election special audit | T+30 to T+180 | OAG commences special audit; evidence produced |
| Management response deadline | T+210 (2028-03-08) | Within 30 days of draft findings |

---

## 14. Dependencies

- **M20, M10, M02/M03** — evidence data sources
- **M15** — IAB published-report cross-reference
- **M58** — parliamentary reporting triggered by OAG findings
- **P05, P06** — audit log and HSM signing
- **IEBC ERP** — external procurement data adapter (post-award specification)
- OAG formal engagement letter — required before scope negotiation begins

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | OAG requests data categories outside agreed scope | Scope-lock mechanism enforces boundary at API level; scope amendment requires new dual sign-off; any out-of-scope request is logged as an access attempt and notified to IEBC DPO |
| R2 | 30-day management response deadline missed | T-14 and T-7 alerts to IEBC CEO; parallel draft preparation starts at draft-finding receipt; Internal Auditor owns deadline tracking |
| R3 | Evidence extract from legacy or offline KIEMS module fails | Module-level extract adapters have 48-hour fallback: manual extract by Datacraft Finance Lead from module backup; logged as manual extraction with explanation |
| R4 | OAG publishes adverse finding that IEBC disputes | Dispute process: formal letter to Auditor-General under Public Audit Act s.32; M58 generates supporting document bundle; record of dispute in M59 follow-up tracker |
| R5 | OAG credential compromised during engagement | Time-limited credentials revocable by IEBC Internal Auditor; all OAG sessions logged; credential revocation logged in P05; new credentials issued after security review |

---

## 16. Cross-References

- **M15** — IAB Dashboard: reads published OAG report metadata
- **M20** — IMS: kit-inventory evidence source
- **M10** — RTS Backend: transmission metadata evidence source
- **M40** — Election Court IT: no overlap; OAG access is separate from petition IT
- **M58** — Parliamentary Reporting: OAG findings may trigger special parliamentary report
- **M22** — Campaign Finance: OAG may audit ECFA enforcement costs

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| Auditor-General access to KIEMS data | FR-M59-02, FR-M59-03 |
| Controlled, scoped evidence production | FR-M59-02, NFR-M59-03 |
| Management response within statutory deadline | FR-M59-05, NFR-M59-06 |
| Published audit report integration | FR-M59-06, FR-M59-08 |
| DPA 2019 compliance for audit data | NFR-M59-04, NFR-M59-05, FR-M59-07 |

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
