# M64 — Voter List Public Inspection

**Module ID:** M64
**Name:** Voter List Public Inspection Portal
**Owner:** Datacraft Limited — Civic Participation Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in the Commission per GCC 15.4

**Strategy traceback:** Elections Act §6A; DPA 2019 §26; T-90 to T-60 statutory window

---

## Purpose

Implements the statutory public-inspection regime under Elections Act §6A. Provides per-station voter rolls for public download in the T-90 to T-60 window, redacted to DPA 2019 §26 lawful basis (name and station visible; ID number and biometric reference suppressed). Accepts objections and registration corrections via a structured intake, tracks each objection through resolution, and enforces the statutory deadline for final determination. A no-action on an objection before T-60 constitutes automatic referral to the Elections Court per §6A(4).

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| Member of public | Unauthenticated search | Look up own registration, verify station |
| Objector (authenticated) | Create objection | File objection against another voter's registration |
| Registration Claimant | Authenticated | File correction claim on own record |
| the Commission Registration Officer | Write (assigned stations) | Review objections, record determinations |
| County Returning Officer | Write (own county) | Counter-sign batch determinations |
| Elections Court | Read-only, on order | Receive auto-referrals from lapsed objections |
| IAB Member | Read-only, all | Monitor objection pipeline compliance |
| Civil Society Observer | Read-only, public endpoint | Download redacted rolls, verify completeness |

---

## Functional Requirements

### FR-64.01 — Per-station voter roll generation
At T-90: the Commission Registration Officer triggers roll generation for each station. Roll contains: voter sequence number, full name, station code, ward, constituency, county. ID number, date of birth, and biometric reference suppressed per DPA 2019 §26. Roll signed with HSM key; SHA-256 published. Available for download within 24 hours of generation.

### FR-64.02 — Public search interface
Unauthenticated voters can search by name and station (or by NIIMS reference at authenticated tier). Search returns: confirmation of registration, station name, ward; no ID number in unauthenticated response. Rate-limited to prevent bulk enumeration. WCAG 2.2 AA; EN and SW.

### FR-64.03 — Objection intake (T-90 to T-60)
Any registered voter may file an objection against another registration within the statutory window. Objection form: objector ID (verified via M04 VPRS API), subject voter details (name, station), ground (not-citizen / under-age / deceased / duplicate / false-address / other), supporting document upload (PDF, max 5 MB). Objection assigned a case number on submission. Acknowledgement SMS + email within 5 minutes.

### FR-64.04 — Correction claim intake
Registered voter may file a correction claim on own record (name spelling, address, station transfer). Claimant verified via biometric (M01 BVR device or M13 app). Correction claim distinct from objection in workflow routing.

### FR-64.05 — Objection routing and determination
Objections routed to the Registration Officer for the subject voter's station. Officer reviews supporting documents, may request additional evidence (72-hour window), records determination (upheld / dismissed / referred-to-court). CRO counter-signs batch determination. Every determination logged with reason text and officer UUID.

### FR-64.06 — Statutory deadline enforcement
System enforces T-60 hard close. Any objection not determined by T-60 23:59 EAT is auto-referred to Elections Court via structured referral packet (objection record, submitted evidence, case number). Auto-referral triggers notification to objector, subject voter's CRO, and the Commission Legal. IAB receives daily lapsed-objection count.

### FR-64.07 — Roll update after determination
Upheld objections trigger a proposed deletion or correction in M04 VPRS staging table. Deletions and corrections require dual-officer approval (Registration Officer + CRO) before commit. Committed changes: new signed roll version generated and published within 2 hours.

### FR-64.08 — Statutory-period compliance dashboard
Internal dashboard showing: days remaining in inspection window, objection counts by county (filed / pending / determined / lapsed), correction claim counts, on-time determination rate. IAB read access. Auto-alert to the Commission Chair if any county's lapsed rate exceeds 5%.

### FR-64.09 — Roll integrity publication
Each station roll version: SHA-256, generation timestamp, signer key fingerprint, record count. Published to the Commission public portal and mirrored on IAB domain. Any independently computed hash mismatch triggers P0 alert.

---

## Non-Functional Requirements

### NFR-64.01 — WCAG 2.2 AA
All voter-facing screens (search, objection intake, claim intake) meet WCAG 2.2 AA. Screen-reader compatible. Mobile-first layout (55% of Kenyan internet access is mobile). EN and SW throughout.

### NFR-64.02 — Availability
99.9% during T-90 to T-60 statutory window. Public search read endpoint served from CDN.

### NFR-64.03 — DPA 2019 lawful basis
Processing for public inspection: Elections Act §6A as public-task lawful basis per DPA 2019 §30(1)(e). DPIA completed. ID numbers and biometric references not disclosed. Access logs retained 7 years per Elections Act §44.

### NFR-64.04 — Rate limiting
Unauthenticated search: 10 requests/minute per IP. Download endpoint: 5 downloads/hour per IP. Bot-pattern detection; CAPTCHA at threshold.

### NFR-64.05 — Scale
54,193,665 registered voters across 55,393 stations; average 979 voters/station. National roll compressed < 2 GB (CSV + PDF per station). Station-level downloads < 500 KB each.

### NFR-64.06 — SLA
Objection acknowledgement: < 5 minutes. Officer determination response window: 7 days from filing (system enforced). Auto-referral to Elections Court: within 1 hour of T-60 deadline breach.

---

- **NFR-M64-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-M64`; alert ruleset ID `iems-M64-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As a voter, I want to search my name and confirm which station I am registered at, so that I know where to vote on election day.
2. As a Kenyan citizen, I want to download the voter roll for my polling station, so that I can verify all listed persons are eligible.
3. As an objector, I want to file an objection online against a registration I believe is fraudulent, so that the Commission can investigate before the roll is finalised.
4. As a voter whose name is misspelled, I want to file a correction claim and have it resolved before T-60, so that my registration is accurate on election day.
5. As a Registration Officer, I want to see all open objections for my stations in a prioritised queue, so that I can meet the T-60 determination deadline.
6. As a County Returning Officer, I want to counter-sign batch determinations for my county, so that the dual-approval requirement is met before any roll changes are committed.
7. As an IAB Member, I want to see the daily lapsed-objection count per county, so that I can flag systemic non-compliance before the statutory window closes.
8. As an Elections Court Registrar, I want to receive a structured auto-referral packet for each lapsed objection, so that the court can proceed without waiting for the Commission to submit manually.
9. As a civil society observer, I want to download the signed station roll and verify its SHA-256, so that I can confirm the Commission has not modified the roll after publication.
10. As the Commission Chair, I want an alert if any county's lapsed-objection rate exceeds 5%, so that I can intervene with additional officers before the statutory deadline.
11. As a voter with a disability, I want the search and objection forms to work with my screen reader, so that the inspection process is equally accessible to me.
12. As a data journalist, I want the national roll record counts per station to be downloadable as CSV, so that I can independently verify total registration numbers.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict, Annotated
from datetime import datetime
from enum import Enum

class RollVersion(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    version: int
    record_count: int
    sha256: str
    generated_at: datetime
    signed_by: str  # HSM key fingerprint
    download_url: str
    is_current: bool

class Objection(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7 — case number
    objector_id: str  # verified voter UUID
    subject_name: str
    subject_station_code: str
    ground: str  # not_citizen | under_age | deceased | duplicate | false_address | other
    evidence_urls: list[str]
    filed_at: datetime
    status: str  # pending | under_review | determined | auto_referred
    determination: str | None  # upheld | dismissed | referred_to_court
    determination_reason: str | None
    officer_id: str | None
    officer_determined_at: datetime | None
    cro_signature: str | None
    referral_packet_url: str | None

class CorrectionClaim(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    claimant_id: str  # verified voter UUID
    field: str  # name | address | station
    current_value: str
    requested_value: str
    evidence_url: str | None
    filed_at: datetime
    status: str  # pending | approved | rejected
    officer_id: str | None
    approved_at: datetime | None

class RollChange(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    objection_id: str | None
    correction_id: str | None
    change_type: str  # deletion | correction
    station_code: str
    proposed_by: str  # officer UUID
    approved_by: str  # CRO UUID
    committed_at: datetime | None
    new_roll_version_id: str | None
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M64 Voter List Public Inspection API
  version: "1.0.0"
paths:
  /api/v1/voter-list/search:
    get:
      summary: Public voter registration search (unauthenticated)
      parameters:
        - name: name
          in: query
          required: true
          schema: {type: string}
        - name: station_code
          in: query
          schema: {type: string}
      responses:
        "200": {description: Registration confirmation (no ID number)}
        "429": {description: Rate limit exceeded}
  /api/v1/voter-list/rolls/{station_code}/current:
    get:
      summary: Download current signed station roll (redacted)
      responses:
        "200":
          description: Signed roll PDF or CSV
          content:
            text/csv:
              schema: {type: string}
  /api/v1/voter-list/objections:
    post:
      summary: File objection (authenticated voter)
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/Objection"}
      responses:
        "201": {description: Objection created; case number returned}
  /api/v1/voter-list/objections/{id}:
    get:
      summary: Get objection status
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Objection record}
    patch:
      summary: Record determination (Registration Officer)
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {type: object, properties: {determination: {type: string}, reason: {type: string}}}
      responses:
        "200": {description: Determination recorded}
  /api/v1/voter-list/corrections:
    post:
      summary: File correction claim (authenticated voter)
      security: [{BearerAuth: []}]
      responses:
        "201": {description: Correction claim created}
  /api/v1/voter-list/compliance-dashboard:
    get:
      summary: Statutory period compliance metrics
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Compliance metrics by county}
```

---

## Integrations

**Upstream:**
- M04 (VPRS) — voter registration master data; staged roll extract; correction commit
- M01 (BVR Device) — biometric verification for correction claimants
- M13 (Public Verifier App) — voter self-lookup integration
- P04 (IAM) — officer authentication, role scoping
- P06 (HSM) — roll signing

**Downstream:**
- M10 (RTS Back-End) — finalised roll snapshot at T-60 feeds into polling-day authentication
- M15 (IAB Dashboard) — compliance metrics feed
- Elections Court API — structured auto-referral packets
- M11 (Public Portal) — roll download links surfaced to public
- P05 (Audit log) — all determination and change events

---

## UI/UX — Key Screens

1. **Public search page** — single input (name), optional station filter; confirmation card with station name and ward; no ID returned; EN/SW toggle; WCAG 2.2 AA; mobile-first
2. **Station roll download page** — list of all stations in selected county; per-station signed roll link + SHA-256 displayed; CSV and PDF options
3. **Objection intake form** — step-by-step; ground selection with plain-language descriptions; document upload; case number displayed on confirmation; SMS acknowledgement
4. **Correction claim form** — biometric-verified; current value pre-filled; requested value + evidence upload
5. **Registration Officer queue** — sorted by deadline proximity; objection detail; determination form; document viewer; request-evidence button
6. **CRO county view** — officer queues across stations; batch counter-sign workflow; compliance bar chart
7. **Compliance dashboard** (the Commission internal + IAB) — countdown to T-60; lapsed-objection heat map; county-level drill-down

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Statutory public inspection right | Elections Act §6A |
| T-90 to T-60 inspection window | Elections Act §6A(1) |
| Auto-referral on lapsed objection | Elections Act §6A(4) |
| DPA 2019 redaction (ID, biometric) | DPA 2019 §26; public-task lawful basis §30(1)(e) |
| Records retention | Elections Act §44 (7 years) |
| DPIA requirement for public roll | DPA 2019 §31 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Objection acknowledgement | p99 < 5 min | the Commission ICT |
| On-time determination rate (before T-60) | >= 95% | the Commission Chair |
| Auto-referral delivery to Elections Court | p99 < 1h of T-60 breach | the Commission Legal |
| Roll SHA-256 independent verification success | 100% | IAB |
| WCAG 2.2 AA audit pass | 100% | Accessibility auditor |
| Portal uptime during statutory window | >= 99.9% | Datacraft SRE |

---

## Test Strategy

- **Unit:** Statutory window enforcement, auto-referral trigger, redaction logic, roll SHA-256 computation
- **Integration:** M04 VPRS staged roll extract, M01 biometric verification, Elections Court referral packet delivery, HSM roll signing
- **Load:** 1 million public searches in first 24 hours of T-90; p99 < 2s
- **Accessibility:** Automated axe-core + manual screen-reader test on objection and search flows
- **DPA compliance:** Automated scan confirming ID numbers absent from all public API responses
- **Dry run:** Full objection-to-determination cycle at T-150 (mock election)

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| M04 VPRS roll-export API agreed | 2027-01-15 | API contract |
| Portal and intake forms built | 2027-03-01 | Internal UAT |
| DPIA completed and ODPC acknowledged | 2027-04-01 | Compliance gate |
| Elections Court integration tested | 2027-05-01 | Referral packet accepted |
| Statutory window opens | 2027-05-11 (T-90) | Live |
| Statutory window closes | 2027-06-10 (T-60) | Final rolls locked |

---

## Dependencies

- **M04** (VPRS) — registration master data
- **M01** (BVR) — biometric verification
- **M10** (RTS Back-End) — final roll snapshot at T-60
- **M15** (IAB Dashboard) — compliance feed
- **P04** (IAM), **P05** (Audit log), **P06** (HSM)

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Bulk enumeration via public search | Rate limiting, CAPTCHA, bot-pattern detection; search returns per-voter confirmation only, not list responses |
| R2 | Officers unable to clear objection backlog before T-60 | Daily lapsed-rate alert from T-75; the Commission Chair intervention protocol; additional officers deployable per county |
| R3 | Correction claim commits bad data to M04 VPRS | Dual-officer approval gate; staged-table pattern; no direct write to live roll |
| R4 | Elections Court auto-referral packet rejected on format | Pre-agreed XML schema with Judiciary IT; tested at T-150 mock election |
| R5 | High-volume roll downloads overload origin servers | CDN-cached signed rolls; origin only serves signed manifest; CDN invalidated on roll version update |

---

## Cross-References

- **M04** VPRS — registration master; correction target
- **M01** BVR — biometric correction-claim verification
- **M11** Public Portal — roll download surfacing
- **M13** Public Verifier App — voter self-lookup
- **M15** IAB Dashboard — compliance metrics
- **M10** RTS — final roll snapshot at T-60

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| Elections Act §6A | Statutory public inspection, T-90 to T-60 window |
| Elections Act §6A(4) | Auto-referral to Elections Court on lapsed objection |
| DPA 2019 §26, §30(1)(e) | Lawful basis and redaction obligations |
| DPA 2019 §31 | DPIA requirement |
| Elections Act §44 | 7-year record retention |

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
