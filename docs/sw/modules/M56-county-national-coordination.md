# M56 — County and National Government Coordination

**Module ID:** M56
**Name:** County and National Government Coordination Interface
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in the Commission per GCC 15.4

**Strategy traceback:** Part II §PM-09, Part V §V.3, Part IX §IX.2.G, Part XII §XII.2, Part XXI.A.15

---

## 1. Purpose

Devolution under Constitution Art. 6 and the County Governments Act 2012 means the Commission depends on county governments for ground-level logistics data (demography, road networks, health facilities, security hotspots, school availability as polling station hosts) and on national-government line ministries (Interior, Health, Education, Transport) for statutory approvals and support. This module provides a structured, audited data-exchange and coordination portal so that inter-governmental dependencies are tracked, MoUs are registered, and cross-jurisdiction incident coordination during election operations is operationally reliable. It replaces informal email chains with a workflow-driven record that feeds directly into M16 (Election Command Centre) situational awareness.

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| the Commission County Coordinator | Lead the Commission officer per county | Full RW on county coordination workspace |
| County Elections Officer (CEO) | Senior county government liaison | Read county data; submit data contributions |
| County Government Analyst | Data officer from county | Upload demographic / facility data |
| National MoU Registry Officer | the Commission HQ tracking officer | MoU lifecycle management |
| Ministry Line Officer | Interior / Health / Education / Transport officer | Submit approvals; view shared data |
| the Commission Director of Operations | Oversight | Read-all; approve MoUs |
| M16 Ops Analyst | Election Command Centre | Read county situational data feeds |

---

## 3. Functional Requirements

**FR-M56-01 — MoU registry:** Record, version, and track status of all bilateral MoUs between the Commission and county governments or national-government line ministries. MoU lifecycle: draft, review, signed, active, expired. Signed MoUs stored as HSM-timestamped PDF blobs. Expiry alerts at T-90 and T-30.

**FR-M56-02 — County data-sharing workspace:** Per-county workspace where the county government uploads structured datasets: population per sub-location, school facility capacity (per Education Ministry format), health facility GPS coordinates, road-accessibility index, security-incident history. All uploads validated against published schema (JSON Schema v7) before acceptance.

**FR-M56-03 — Polling-station siting data layer:** Aggregate county-contributed data into a merged layer consumed by the Commission Boundary & Logistics team for polling-station siting decisions. Every data point carries provenance (source county, uploaded by, timestamp, data-schema version). Layer exported to M18 (Boundary Transparency) for public display.

**FR-M56-04 — Ministerial approval tracking:** For each election requiring a national-government approval (security deployment request to Interior, health-unit pre-positioning to Health, Education school-use approval), create a structured approval request with deadline, statutory basis, and escalation path. Track response status. Overdue approvals trigger the Commission Director of Operations alert.

**FR-M56-05 — Incident coordination log:** During election operations, county coordinators and ministry officers log cross-jurisdiction incidents (road closure, health emergency at polling station, security event). Incidents linked to polling-station codes and routed to M16 ECC in real time. Status transitions: open, assigned, resolved. Resolution requires sign-off by both the Commission and county officer.

**FR-M56-06 — Cross-jurisdiction data-sharing audit:** Every data item received from a county or ministry is logged with originating authority, DPA 2019 lawful basis (public task, s.30(1)(d)), purpose code, and retention window. Data shared outward (to M18, M16) is logged with recipient module and justification.

**FR-M56-07 — County coordination calendar:** Shared calendar of joint events: pre-election county briefings, logistic reconnaissance visits, security-joint-committee meetings. Calendar entries have attendee list, minutes-upload slot, and action-item tracker.

**FR-M56-08 — Language support:** All county-facing screens available in EN and SW. County data upload templates provided in EN and SW with field-level guidance.

---

## 4. Non-Functional Requirements

**NFR-M56-01 — Performance:** County workspace page load p99 < 1.5 s under 500 concurrent county users (47 counties, multiple users per county).

**NFR-M56-02 — Reliability:** 99.5% uptime T-180 through T+30. Degraded mode (read-only county workspaces) if central services unavailable.

**NFR-M56-03 — Security:** County government users authenticated via national government IDMS federation (SAML 2.0) or Commission-issued credentials with TOTP. Role isolation: county A cannot view county B workspace. All file uploads virus-scanned before storage.

**NFR-M56-04 — WCAG 2.2 AA:** County-facing portal and ministry approval portal. EN + SW baseline.

**NFR-M56-05 — Scale:** 47 counties, up to 20 ministries, up to 10 users per entity = ~670 external users peak.

**NFR-M56-06 — DPA 2019 lawful basis:** Processing county demographic data under s.30(1)(d) (public interest / public task). DPIA registered with ODPC before T-180. Data minimisation: no individual voter data in county uploads; aggregate sub-location level only.

**NFR-M56-07 — SLA:** MoU expiry alert delivered 100% of the time at T-90 and T-30. Incident routing to M16 ECC p99 < 30 s.

---

- **NFR-56-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-56` (replace 56 with the module number); alert ruleset ID `iems-56-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Commission County Coordinator, I want to see all pending county government data uploads on a single dashboard, so I can chase missing datasets before the siting deadline.
2. As a County Government Analyst, I upload a school-facility CSV and the system validates my data against the published schema and returns field-level errors, so I fix problems before submission.
3. As a Ministry Line Officer, I receive a structured approval request with a statutory deadline, and I submit my response through the portal, so the approval is on record with a timestamp.
4. As the Commission Director of Operations, I view all MoUs expiring within 90 days and initiate a renewal workflow, so no coordination agreement lapses before election day.
5. As an M16 Ops Analyst, I receive county incident events in real time via SSE feed, so I can update the ECC situational picture without manual polling.
6. As a County Elections Officer, I log a road-closure incident linked to three polling stations, so the Commission logistics can reroute and the event is on the official record.
7. As the National MoU Registry Officer, I upload a signed MoU PDF and the system timestamps it and links it to all data-sharing permissions it authorises.
8. As a Commission County Coordinator, I download a provenance-tagged polling-station siting data package for my county to share with the county boundaries committee.
9. As a Ministry of Health officer, I confirm pre-positioning of health units at 12 county venues, and each confirmation is linked to the corresponding approval request.
10. As a Commission data-protection officer, I review the DPA audit log for all county data received in the last 30 days and confirm every item has a valid lawful-basis code.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime, date
from typing import Literal

class CountyMoU(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str()
    party_type: Literal["county_government", "national_ministry"]
    party_code: str          # county FIPS or ministry code
    party_name: str
    mou_title: str
    signed_date: date | None
    effective_from: date | None
    expires_on: date | None
    status: Literal["draft", "under_review", "signed", "active", "expired"]
    signed_pdf_hash: str | None
    created_by: str
    created_at: datetime

class CountyDataUpload(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    county_code: str
    dataset_type: Literal["population", "school_facility", "health_facility", "road_access", "security_history"]
    schema_version: str
    file_hash: str
    uploaded_by: str
    uploaded_at: datetime
    validation_status: Literal["pending", "valid", "invalid"]
    validation_errors: list[str]
    dpa_lawful_basis: str   # e.g. "DPA2019_s30_1_d"
    purpose_code: str
    retention_until: date

class CrossJurisdictionIncident(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    polling_station_codes: list[str]
    county_code: str
    incident_type: Literal["road_closure", "health_emergency", "security_event", "infrastructure", "other"]
    description: str
    reported_by: str
    reported_at: datetime
    status: Literal["open", "assigned", "resolved"]
    commission_signoff: str | None
    county_signoff: str | None
    resolved_at: datetime | None

class MinisterialApproval(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    ministry_code: str
    approval_type: str
    statutory_basis: str
    requested_at: datetime
    deadline: datetime
    status: Literal["pending", "approved", "rejected", "overdue"]
    response_at: datetime | None
    response_by: str | None
    response_notes: str | None
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/county/{code}/workspace` | County JWT | 100/min | County data dashboard |
| POST | `/api/v1/county/{code}/uploads` | County JWT | 20/hr | Submit county dataset |
| GET | `/api/v1/county/{code}/uploads/{id}/validation` | County JWT | 60/min | Validation result |
| GET | `/api/v1/mous` | the Commission JWT | 60/min | List MoUs with status |
| POST | `/api/v1/mous` | COMMISSION_ADMIN JWT | 10/hr | Register new MoU |
| PATCH | `/api/v1/mous/{id}` | COMMISSION_ADMIN JWT | 10/hr | Update MoU status / upload signed PDF |
| GET | `/api/v1/approvals` | COMMISSION_OPS JWT | 60/min | All ministerial approvals |
| POST | `/api/v1/approvals` | COMMISSION_OPS JWT | 30/hr | Create approval request |
| POST | `/api/v1/approvals/{id}/respond` | MINISTRY JWT | 20/hr | Ministry submits response |
| POST | `/api/v1/incidents` | COUNTY JWT | 60/hr | Log cross-jurisdiction incident |
| GET | `/api/v1/incidents/stream` | M16 SERVICE JWT | SSE | Real-time incident feed to ECC |
| GET | `/api/v1/siting-layer/{county_code}` | COMMISSION_OPS JWT | 30/min | Merged siting data for county |
| GET | `/api/v1/dpa/audit` | COMMISSION_DPO JWT | 20/min | DPA audit log |

---

## 8. Integrations

- **M16 Election Command Centre** — real-time incident SSE feed; county situational data layer
- **M18 Boundary Transparency** — siting data export for public display
- **P05 Audit Log** — all data exchanges logged and Merkle-anchored
- **P04 IAM** — county government user federation (SAML 2.0); the Commission staff auth
- **Ministry IDMS** (external) — SAML federation where ministry systems support it
- **ODPC** (external) — DPIA filing; breach notification if county data incident

---

## 9. UI/UX Key Screens

1. **County Coordinator Dashboard** — upload status per dataset type; MoU expiry ticker; open incidents count; calendar of upcoming joint events.
2. **Data Upload Wizard** — file picker, schema-version selector, validation result with field-level error list, DPA purpose-code dropdown before submission.
3. **MoU Registry** — filterable table by party, status, expiry; one-click renewal workflow launch; signed PDF viewer.
4. **Approval Tracker** — kanban: pending / approved / overdue; overdue items highlighted red; one-click escalation email to director.
5. **Incident Log** — map view of open incidents overlaid on county boundaries; status column; sign-off workflow for resolution.
6. **Siting Data Layer Preview** — choropleth of county with data-completeness indicator per sub-location.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| Devolution coordination obligation | Constitution Art. 6, 189; County Governments Act 2012 s.58 |
| Inter-governmental relations | Intergovernmental Relations Act 2012 |
| Polling station data obligations | Elections Act 2011 s.24; Elections Regulations 2012 reg. 44 |
| DPA 2019 lawful basis for demographic data | DPA 2019 s.30(1)(d) |
| Data sharing agreement requirement | DPA 2019 s.43; ODPC Guidelines on Data Sharing 2023 |
| Access to government information | Access to Information Act 2016 s.4 |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| County dataset upload completion by T-90 | 100% of 47 counties, all 5 dataset types | the Commission Director of Operations |
| MoU expiry alert delivery | 100% at T-90 and T-30 from expiry | the Commission MoU Registry Officer |
| Ministerial approval response rate | >= 95% within deadline | the Commission Director of Operations |
| Incident routed to M16 ECC | p99 < 30 s | Datacraft SRE |
| Portal availability T-180 to T+30 | >= 99.5% | Datacraft SRE |
| DPA audit completeness | 100% of data items have lawful-basis code | the Commission DPO |

---

## 12. Test Strategy

- **Unit:** MoU state machine; dataset schema validator; approval-deadline calculator.
- **Integration:** county upload end-to-end to M18 siting layer; incident SSE delivery to M16 ECC.
- **Access control:** county A cannot access county B workspace; ministry officer cannot read county uploads outside their scope.
- **Load:** 47 concurrent county uploads on the dataset deadline day.
- **Chaos:** M16 SSE consumer offline — verify incident queue persists and replays on reconnect.
- **Accessibility:** WCAG 2.2 AA on county-facing portal (axe-core + manual, EN and SW).

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| MoU templates finalised with CRA | T-545 (2026-03-10) | Approved templates |
| County onboarding (the Commission CEOs) | T-365 (2026-10-10) | All 47 county coordinators credentialled |
| Ministry federation tested | T-300 (2026-12-14) | At least Interior + Health + Education live |
| County data upload open | T-180 (2027-02-10) | Schema validation live; T-90 dataset deadline |
| Siting layer exported to M18 | T-90 (2027-05-10) | Boundary committee has complete data |
| Election-day incident coordination | T-0 (2027-08-10) | Live incident log + M16 integration |
| Post-election MoU review | T+30 (2027-09-10) | Lessons-learned report to the Commission |

---

## 14. Dependencies

- **M16** — incident SSE consumer
- **M18** — siting data export
- **P04 IAM** — county and ministry user authentication
- **P05 Audit Log** — data-exchange audit trail
- National government IDMS — SAML federation (the Commission to negotiate post-award)
- Intergovernmental Relations Act 2012 — legal basis for framework

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | County governments decline to share data citing devolution sovereignty | Engage CRA and NCIC pre-award; frame data sharing as counties contributing to their own residents' service delivery; MoU templates reviewed by Council of Governors |
| R2 | Ministry approval timelines exceed statutory deadlines | Build escalation path to the Commission Chair and CS Interior at T-7 from deadline; pre-brief DODI on escalation protocol |
| R3 | County-uploaded data quality too poor for siting decisions | Schema validation catches errors at upload; the Commission County Coordinator can query and return data to county for correction; fallback to prior census data if corrected upload not received |
| R4 | County security incidents not reported in time to affect M16 ECC | Training during T-180 onboarding; fallback phone hotline logged into incident module retroactively by county coordinator |
| R5 | DPA 2019 breach if demographic data miscategorised | Purpose code and lawful basis captured at upload; DPO automated audit report weekly; county data never linked to voter PII in this module |

---

## 16. Cross-References

- **M16** — incident feed consumer; situational awareness
- **M18** — polling-station siting data export
- **M20** — logistics inventory linked to county access data
- **M15** — IAB can query MoU registry for inter-governmental assurance
- **M23** — county-level grievance data may trigger incidents here
- **M60** — health-facility data from this module feeds medical first-response coordination

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| Coordination with county governments for logistics | FR-M56-02, FR-M56-03 |
| National ministry approval tracking | FR-M56-04 |
| Cross-jurisdiction incident management | FR-M56-05 |
| MoU registry for inter-governmental agreements | FR-M56-01 |
| DPA 2019 data-sharing compliance | FR-M56-06, NFR-M56-06 |

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
