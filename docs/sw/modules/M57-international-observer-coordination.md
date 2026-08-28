# M57 — International Observer Coordination Portal

**Module ID:** M57
**Name:** International Observer Coordination Portal
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part II §PM-10, Part IX §IX.2.F4, Part XII §XII.3, Part XXI.A.16

---

## 1. Purpose

Accredited international observer missions require structured coordination distinct from M43 (domestic accreditation) and M15 (IAB Dashboard). EU EOM, African Union, Carter Center, IFES, Commonwealth Secretariat, and EAC each send missions of 10-200+ observers. IEBC must brief each mission, manage their deployment schedules, receive structured preliminary and final observation reports, and coordinate joint communications without leaking operational security. This module provides a mission-specific coordination workspace: deployment planning, briefing calendar, official report intake, and a joint-communication drafting protocol. It creates a permanent, audited record of every formal interaction between IEBC and each accredited mission — reducing the risk that informal channels produce conflicting public statements.

Distinct from M43: M43 handles domestic party agents and civil-society observers. M57 is exclusively for inter-governmental and international NGO missions that hold observer agreements signed at the level of IEBC Chair or Commissioner.

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| IEBC International Affairs Officer | IEBC coordinator for all missions | Full RW on all mission workspaces |
| Mission Chief Observer | Head of each observer mission | RW on own mission workspace; read briefing materials |
| Mission Coordinator | Mission logistics officer | RW on deployment plan; read briefings |
| IEBC Communications Director | Joint statement review | Read all; approve joint statements |
| IAB Secretariat | Cross-reference to M15 | Read mission report summaries only |
| IEBC Chair / Commissioner | Formal relationship holder | Read-all; sign mission agreements |
| M16 Ops Analyst | ECC situational feed | Read deployment locations (no report content) |

---

## 3. Functional Requirements

**FR-M57-01 — Mission registry:** Register each accredited observer mission with: organisation name, mission type (EU EOM / AU / Carter Center / IFES / Commonwealth / EAC / Other), mandate dates, Chief Observer identity, IEBC counterpart assignment, and signed mission agreement PDF (HSM-timestamped). Mission status lifecycle: pending, credentialled, deployed, post-mission.

**FR-M57-02 — Deployment coordination:** Per-mission deployment plan: number of observers, deployment regions (county-level), arrival and departure schedule, vehicle registration register, accommodation details. Deployment data exported to M16 ECC as a read-only situational-awareness overlay (no report content). Changes to deployment plans trigger notification to IEBC International Affairs Officer.

**FR-M57-03 — Briefing calendar:** IEBC publishes structured briefing events to all accredited missions simultaneously: pre-election technical briefing, election-day operations briefing, results-transmission briefing. Each briefing has: date/time, venue, agenda PDF, attendance register, minutes upload slot. Observers can submit questions before briefings; answers added to briefing record.

**FR-M57-04 — Structured report intake:** Formal intake of Preliminary Statements, Interim Reports, and Final Reports from each mission. Each report submission: mission ID, report type, report date, PDF upload (HSM-timestamped), structured metadata (reporting period, preliminary findings category). IEBC acknowledges receipt within 6 hours. Reports stored permanently; cross-referenced to M15 IAB quarterly report cycle.

**FR-M57-05 — Joint communication protocol:** Workflow for joint IEBC-Observer statements. Initiating party (IEBC or mission) drafts statement in shared workspace. IEBC Communications Director and Mission Chief Observer both must approve before publication. Version history maintained. Publication event logged with both parties' approval timestamps.

**FR-M57-06 — Mission portal (read-only public summary):** Public-facing page at `iebc.go.ke/observers` listing all accredited missions, mission type, mandate dates, and links to published Final Reports. No deployment details or preliminary report contents until mission publishes them.

**FR-M57-07 — Audit trail:** All interactions (briefing attendance, report submissions, joint-statement approvals, deployment changes) logged in P05 with actor, timestamp, and event type. IAB has read access to the log (not report content) via M15.

**FR-M57-08 — Multilingual support:** Portal EN + SW. Mission briefing materials translated to FR and Arabic where mission mandate requires (EU EOM, AU missions). Mission Chief Observers can submit reports in EN or FR; system preserves original and tags language.

---

## 4. Non-Functional Requirements

**NFR-M57-01 — Performance:** Mission workspace p99 < 1.5 s; briefing calendar p99 < 1 s.

**NFR-M57-02 — Reliability:** 99.5% uptime T-365 through T+90. Mission portal must remain online during petition window (T+90 at minimum).

**NFR-M57-03 — Security:** Mission users authenticated with IEBC-issued TOTP credentials. Each mission's workspace is strictly isolated. Joint-statement workspace visible to both parties' approved users only. IEBC Communications Director approval required before any cross-mission communication.

**NFR-M57-04 — WCAG 2.2 AA:** All mission-facing screens and public observer portal.

**NFR-M57-05 — Scale:** Up to 15 concurrent observer missions, each with up to 30 user accounts = ~450 external users peak.

**NFR-M57-06 — DPA 2019 lawful basis:** Observer personal data (name, role, nationality, deployment location) processed under s.30(1)(d) (public interest). Retention: 7 years post-election. Observer reports treated as institutional documents; no individual voter PII in scope.

**NFR-M57-07 — SLA:** Report receipt acknowledgement within 6 hours. Briefing calendar published >= 30 days before election day. Joint statement approval SLA: 24 hours from initiating party submission.

---

- **NFR-57-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-57` (replace 57 with the module number); alert ruleset ID `iems-57-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the IEBC International Affairs Officer, I register the EU EOM mission and upload the signed mission agreement, so the mission's workspace is active and the agreement is timestamped.
2. As a Mission Coordinator from the African Union, I update our deployment plan with 15 new regional assignments, and the system notifies the IEBC counterpart within 5 minutes.
3. As the IEBC Communications Director, I review a joint statement draft from the Carter Center and approve it; both approval timestamps are recorded before publication.
4. As a Mission Chief Observer, I submit our Preliminary Statement PDF and receive an automated acknowledgement within 6 hours.
5. As the IAB Secretariat, I view the list of all mission reports received and their submission dates, so I can cross-reference them in the IAB quarterly report.
6. As a member of the public, I access the public observer portal and download the EAC mission's Final Report from the published list.
7. As an M16 Ops Analyst, I see the EU EOM's deployment regions on the ECC county map, so I can ensure polling-station access is coordinated.
8. As the IEBC International Affairs Officer, I publish the pre-election technical briefing agenda to all 12 accredited missions simultaneously with one action.
9. As a Mission Coordinator, I submit pre-briefing questions through the portal, and the answers appear in the briefing record after the event.
10. As the IEBC Chair, I sign the Commonwealth Secretariat mission agreement digitally and it is linked to the mission registry entry.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime, date
from typing import Literal

class ObserverMission(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str()
    organisation_name: str
    mission_type: Literal["EU_EOM","AU","CARTER_CENTER","IFES","COMMONWEALTH","EAC","OTHER"]
    mandate_start: date
    mandate_end: date
    chief_observer_name: str
    chief_observer_email: str
    iebc_counterpart_id: str
    agreement_pdf_hash: str | None
    status: Literal["pending","credentialled","deployed","post_mission"]
    created_at: datetime

class DeploymentPlan(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    mission_id: str
    observer_count: int
    regions: list[str]       # county codes
    arrival_date: date
    departure_date: date
    version: int
    updated_by: str
    updated_at: datetime

class BriefingEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    title: str
    briefing_type: Literal["pre_election_technical","election_day_ops","results_transmission","other"]
    scheduled_at: datetime
    venue: str
    agenda_pdf_hash: str | None
    minutes_pdf_hash: str | None
    invitees: list[str]      # mission IDs
    attendees: list[str]     # mission IDs confirmed
    questions: list[dict]    # {"mission_id": str, "question": str, "answer": str | None}

class MissionReport(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    mission_id: str
    report_type: Literal["preliminary_statement","interim_report","final_report"]
    reporting_period_start: date
    reporting_period_end: date
    language: Literal["EN","FR","SW","AR"]
    pdf_hash: str
    submitted_at: datetime
    acknowledged_at: datetime | None
    iebc_acknowledged_by: str | None

class JointStatement(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    mission_id: str
    initiated_by: Literal["IEBC","MISSION"]
    draft_text: str
    version: int
    iebc_approved_by: str | None
    iebc_approved_at: datetime | None
    mission_approved_by: str | None
    mission_approved_at: datetime | None
    published_at: datetime | None
    published_url: str | None
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/missions` | IEBC JWT | 60/min | List all missions |
| POST | `/api/v1/missions` | IEBC_ADMIN JWT | 5/hr | Register mission |
| PATCH | `/api/v1/missions/{id}` | IEBC_ADMIN JWT | 20/hr | Update mission status / agreement |
| GET | `/api/v1/missions/{id}/deployment` | MISSION JWT | 60/min | View own deployment plan |
| PUT | `/api/v1/missions/{id}/deployment` | MISSION JWT | 10/hr | Update deployment plan |
| GET | `/api/v1/briefings` | IEBC or MISSION JWT | 60/min | List briefing events |
| POST | `/api/v1/briefings` | IEBC_INTL JWT | 5/hr | Create briefing event |
| POST | `/api/v1/briefings/{id}/questions` | MISSION JWT | 20/hr | Submit pre-briefing question |
| POST | `/api/v1/missions/{id}/reports` | MISSION JWT | 10/hr | Submit mission report |
| GET | `/api/v1/missions/{id}/reports` | IEBC_INTL JWT | 60/min | List mission reports |
| POST | `/api/v1/missions/{id}/statements` | IEBC or MISSION JWT | 5/hr | Initiate joint statement |
| POST | `/api/v1/missions/{id}/statements/{sid}/approve` | IEBC_COMMS or MISSION JWT | 10/hr | Approve joint statement |
| GET | `/api/v1/public/missions` | Public | 200/min | Public mission list + published reports |
| GET | `/api/v1/deployments/map` | M16 SERVICE JWT | SSE | Deployment overlay for ECC |

---

## 8. Integrations

- **M15 IAB Dashboard** — audit log read access; cross-reference to mission report submission timeline
- **M16 Election Command Centre** — deployment location overlay (read-only, no report content)
- **M43 Domestic Accreditation** — M57 is strictly separate; no crossover of mission and domestic observer records
- **P05 Audit Log** — all interactions logged and Merkle-anchored
- **P06 HSM** — mission agreement PDF timestamping; IEBC Chair digital signature on agreements
- **P04 IAM** — mission user credential issuance; TOTP enrolment

---

## 9. UI/UX Key Screens

1. **IEBC International Affairs Dashboard** — mission status cards; upcoming briefings; recent report submissions; pending joint-statement approvals.
2. **Mission Workspace** — mission profile; deployment plan editor; briefing calendar; report submission history; joint-statement drafts.
3. **Briefing Event Page** — agenda, venue, attendance register, Q&A thread, minutes upload.
4. **Report Submission Flow** — report type selector, date range, PDF upload, language tag, submission confirmation with receipt token.
5. **Joint Statement Workflow** — draft editor with tracked changes; approval status for IEBC and mission; publish action (both parties approved).
6. **Public Observer Portal** — mission list with organisation name, mandate dates, published reports as downloadable PDFs.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| International observer access obligation | African Charter on Democracy 2007 Art. 22; AU Guidelines on Election Observation 2002 |
| IEBC obligation to cooperate with observer missions | Elections Act 2011 s.81 |
| Freedom of expression and public information | Constitution Art. 33, 35 |
| DPA 2019 lawful basis for observer personal data | DPA 2019 s.30(1)(d) |
| Audit and records retention | Public Archives and Documentation Service Act Cap 19 |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| Report acknowledgement within 6 hours | 100% | IEBC International Affairs Officer |
| Briefing calendar published >= T-30 | 100% | IEBC International Affairs Officer |
| Joint statement approval within 24 hours of initiation | >= 90% | IEBC Communications Director |
| Mission workspace availability T-365 to T+90 | >= 99.5% | Datacraft SRE |
| Deployment overlay delivered to M16 | p99 < 60 s on update | Datacraft SRE |

---

## 12. Test Strategy

- **Unit:** Mission lifecycle state machine; joint-statement approval gate (both parties required).
- **Integration:** Report PDF intake to P05 log anchor; deployment plan SSE to M16 ECC.
- **Access control:** Mission A cannot view mission B workspace; IEBC Comms cannot modify mission reports.
- **E2E:** Full mission lifecycle from registration through final report submission.
- **Security:** Attempt to bypass joint-statement dual-approval; attempt to access deployment data without M16 service JWT.
- **Accessibility:** WCAG 2.2 AA on mission portal and public page.

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| Mission agreement templates approved | T-365 (2026-10-10) | Legal sign-off |
| First mission onboarded (pilot: EAC) | T-270 (2027-01-13) | EAC workspace live |
| All missions credentialled | T-90 (2027-05-10) | All 8-15 missions active |
| Pre-election technical briefing | T-30 (2027-07-11) | Briefing event + Q&A published |
| Election-day deployment overlay live | T-0 (2027-08-10) | M16 integration confirmed |
| Post-mission final reports intake | T+30 to T+90 | All final reports received and acknowledged |

---

## 14. Dependencies

- **M15** — IAB audit log cross-reference
- **M16** — deployment overlay consumer
- **P04 IAM, P05, P06** — auth, audit, signing
- External: signed mission agreements with each organisation (IEBC to negotiate)

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Mission declines to use structured portal, submits reports via email | IEBC International Affairs Officer manually ingests emailed reports via admin upload; portal remains the system of record; encourage via briefing protocol |
| R2 | Joint statement disagreement delays publication | Both-party approval model documents the stalemate; IEBC may publish unilateral statement clearly labelled as such; log captures mission's non-approval |
| R3 | Preliminary Statement content leaked before mission publication | Mission workspace strictly isolated; no IEBC staff can read preliminary report until mission marks it published; DLP scanning on uploads |
| R4 | Deployment information used to target observers | Deployment data shared with M16 at county level only; no GPS or accommodation details in ECC overlay; security briefing for missions on operational risk |
| R5 | Portal unavailable during election day | 99.5% SLA with offline briefing pack fallback (PDF + WhatsApp group managed by International Affairs Officer) |

---

## 16. Cross-References

- **M15** — IAB Dashboard: mission report submission timeline, audit log
- **M16** — ECC: deployment overlay
- **M43** — Domestic accreditation: strictly separate; no record overlap
- **M23** — Grievance Channels: observer complaints routed to M23 if individual rather than formal mission report

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| International observer mission coordination | FR-M57-01, FR-M57-02, FR-M57-03 |
| Formal report intake with chain of custody | FR-M57-04, NFR-M57-07 |
| Joint communication protocol | FR-M57-05 |
| Public transparency of observer presence | FR-M57-06 |
| DPA 2019 compliance for observer data | NFR-M57-06 |

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
