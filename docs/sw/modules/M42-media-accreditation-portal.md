# M42 — Media Accreditation Portal

**Module ID:** M42
**Name:** Media Accreditation Portal
**Runs on:** Central servers (Kenyan cloud) + IEBC back-office
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in IEBC per GCC 15.4

---

## 1. Purpose

Manage accreditation of journalists, photographers, broadcasters, and live-feed operators for election-period coverage. Issues tamper-evident digital credentials, controls access to results centres and restricted polling areas, enforces the Media Council Act 2013 code-of-conduct obligations, and provides a revocation workflow for accreditation breaches.

Strategy reference: §XVIII (Stakeholder Engagement). Elections Act s.21A; Media Council of Kenya Act 2013 s.29; IEBC Media Accreditation Regulations.

---

## 2. User Personas

- **Journalist / Photographer** — applies for individual accreditation; downloads digital badge; manages credential renewal
- **Broadcaster Representative** — applies for crew accreditation and live-feed access permits; manages team roster
- **IEBC Media Officer** — reviews applications, conducts vetting, issues or rejects credentials, manages revocations
- **Media Council of Kenya (MCK) Liaison** — provides membership status via API; receives breach referrals
- **Results Centre Manager** — scans badges at entry points; views real-time accredited-person counts per centre
- **IEBC Security Officer** — initiates emergency revocation; views live presence dashboard
- **Presiding Officer** — scans badge at polling station for permitted-media list

---

## 3. Functional Requirements

- **FR-M42-001** Online application form: full name, NIN/Passport, employer organisation, MCK membership number, coverage scope (national/county/constituency), access zones requested, photo (ICAO-compliant).
- **FR-M42-002** MCK membership verification via MCK API; flag non-MCK applicants for enhanced vetting.
- **FR-M42-003** Background vetting checklist: NPS clearance check (advisory API), IEBC internal watch-list cross-reference, prior accreditation revocation history.
- **FR-M42-004** Application lifecycle: SUBMITTED → UNDER_VETTING → APPROVED / REJECTED → BADGE_ISSUED → ACTIVE / SUSPENDED / REVOKED.
- **FR-M42-005** Bulk organisation onboarding: broadcaster submits crew roster (CSV/XLSX); system generates individual applications per crew member; organisation admin tracks status per person.
- **FR-M42-006** Digital badge issuance: QR-code-bearing PDF badge with holder name, photo, organisation, access zones, validity dates, badge serial (UUID7); signed with IEBC HSM key; verifiable offline via embedded signature.
- **FR-M42-007** Live-feed access permit: separate permit layer for broadcast crews needing fibre/VSAT access at results centres; permit linked to accreditation record; integrates with M16 Election Command Centre network-access list.
- **FR-M42-008** Code-of-conduct acceptance: applicant digitally acknowledges IEBC media code of conduct before badge issuance; acknowledgement timestamped and stored.
- **FR-M42-009** Badge scanning at entry points: mobile scanner app (QR) confirms accreditation status, zone access, and counts presence at each results centre in real time.
- **FR-M42-010** Revocation workflow: IEBC Media Officer or Security Officer initiates revocation with reason code (BREACH / MISCONDUCT / SECURITY / EXPIRED); badge serial added to revocation CRL published every 5 minutes; offline scanners sync CRL at each scan session.
- **FR-M42-011** Breach referral to MCK: on BREACH revocation, generate MCK referral package (signed PDF with badge serial, timestamp, breach description, officer details); transmitted via MCK API.
- **FR-M42-012** Accreditation window enforcement: portal accepts applications only within the gazetted accreditation window; late applications rejected with reason and statutory reference.
- **FR-M42-013** Public accredited-organisation list: IEBC publishes organisation names (not individual journalists) of accredited media houses; updated daily during active window.
- **FR-M42-014** Every accreditation action audit-logged with actor, timestamp, IP, and purpose code.

---

## 4. Non-Functional Requirements

- **NFR-M42-001** Application portal p99 <= 1 s under 2,000 concurrent users.
- **NFR-M42-002** Badge QR verification at entry point: <= 300 ms response including CRL check.
- **NFR-M42-003** CRL refresh: published to CDN every 5 minutes; offline scanners sync on connection.
- **NFR-M42-004** WCAG 2.2 AA for all applicant-facing screens; EN + SW baseline.
- **NFR-M42-005** Availability >= 99.9% during accreditation window and election day.
- **NFR-M42-006** DPA 2019 s.30(1)(b) lawful basis (legal obligation); biometric photo retained 7 years; NPS advisory data not stored beyond vetting session; DPIA registered with ODPC.
- **NFR-M42-007** RBAC: Media Officer cannot self-approve own organisation; Security Officer revocation triggers a second-officer confirmation for non-emergency events.
- **NFR-M42-008** Audit log immutable; appended to P05 Merkle chain.

---

- **NFR-42-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-42` (replace 42 with the module number); alert ruleset ID `iems-42-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a journalist, I submit my accreditation application online with my MCK number and receive a digital badge PDF within 48 hours of approval.
2. As a broadcaster, I upload a crew roster CSV and track each crew member's accreditation status from one dashboard.
3. As an IEBC Media Officer, I see a vetting checklist with MCK verification status and prior revocation history for each applicant before approving.
4. As a Results Centre Manager, I scan a journalist's badge QR at the door and my scanner shows their permitted zones and current validity in under one second.
5. As an IEBC Security Officer, I revoke a badge immediately on a security incident; the CRL updates within 5 minutes and all entry scanners reflect the revocation at next sync.
6. As a journalist whose application was rejected, I receive a written reason citing the specific regulation that was not satisfied.
7. As a broadcaster technician, I hold a live-feed access permit linked to my accreditation; Results Centre IT staff see me on the network-access list automatically.
8. As the MCK Liaison, I receive a structured breach referral within 1 hour of any BREACH revocation, with the badge serial and incident description.
9. As a member of the public, I can view the list of accredited media organisations on the IEBC portal without disclosing individual journalist names.
10. As an IEBC Commissioner, I can pull a real-time count of accredited journalists present at each results centre on election day.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict, Annotated
from datetime import date, datetime
from enum import Enum
from typing import Optional

class AccreditationStatus(str, Enum):
    SUBMITTED = "SUBMITTED"
    UNDER_VETTING = "UNDER_VETTING"
    APPROVED = "APPROVED"
    REJECTED = "REJECTED"
    BADGE_ISSUED = "BADGE_ISSUED"
    ACTIVE = "ACTIVE"
    SUSPENDED = "SUSPENDED"
    REVOKED = "REVOKED"

class AccessZone(str, Enum):
    POLLING_STATION = "POLLING_STATION"
    TALLYING_CENTRE = "TALLYING_CENTRE"
    NATIONAL_RESULTS_CENTRE = "NATIONAL_RESULTS_CENTRE"
    RESTRICTED_BRIEFING = "RESTRICTED_BRIEFING"

class MediaAccreditation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    applicant_name: str
    id_number: str
    organisation_id: str             # FK to MediaOrganisation
    mck_membership_number: str | None
    photo_url: str
    access_zones: list[AccessZone]
    status: AccreditationStatus
    badge_serial: str | None         # UUID7, set on BADGE_ISSUED
    badge_issued_at: datetime | None
    valid_from: date
    valid_to: date
    revocation_reason: str | None
    revoked_at: datetime | None
    created_at: datetime
    updated_at: datetime

class MediaOrganisation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    name: str
    organisation_type: str           # PRINT / BROADCAST / DIGITAL / AGENCY
    mck_registered: bool
    live_feed_permit: bool
    created_at: datetime
```

---

## 7. API (OpenAPI 3.1 excerpt)

```yaml
openapi: "3.1.0"
info:
  title: M42 Media Accreditation Portal API
  version: "1.0.0"
paths:
  /api/v1/accreditations:
    post:
      summary: Submit accreditation application
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AccreditationApplication'
      responses:
        "202": { description: "Application accepted" }
        "409": { description: "Duplicate application" }
        "422": { description: "Validation error" }
  /api/v1/accreditations/{id}/status:
    get:
      summary: Get accreditation status
      responses:
        "200": { description: "AccreditationStatus object" }
  /api/v1/accreditations/{id}/revoke:
    post:
      summary: Revoke accreditation
      security: [{ BearerAuth: [media_officer, security_officer] }]
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                reason_code: { type: string, enum: [BREACH, MISCONDUCT, SECURITY, EXPIRED] }
                description: { type: string }
      responses:
        "200": { description: "Revocation recorded" }
  /api/v1/badges/{serial}/verify:
    get:
      summary: Verify badge at entry point (offline-capable)
      responses:
        "200": { description: "Badge valid, zones, holder summary" }
        "410": { description: "Badge revoked" }
  /api/v1/crl:
    get:
      summary: Download current Certificate Revocation List
      responses:
        "200":
          content:
            application/json:
              schema:
                type: object
                properties:
                  revoked_serials: { type: array, items: { type: string } }
                  generated_at: { type: string, format: date-time }
```

---

## 8. Integrations

| System | Direction | Purpose |
|---|---|---|
| MCK API | Outbound | Membership verification; breach referral |
| NPS API | Outbound | Advisory clearance check |
| M16 Election Command Centre | Outbound | Network-access list for live-feed permits |
| P05 Merkle / Audit Log | Outbound | Immutable audit trail |
| P06 HSM | Outbound | Badge digital signature |
| M43 Observer Accreditation | Internal | Shared vetting infrastructure |

---

## 9. UI/UX

- Applicant portal: step-form wizard (4 steps), progress indicator, document upload with file-type validation, status-tracking dashboard with email + SMS notifications.
- IEBC back-office: vetting queue with MCK status chips, checklist panel, approve/reject modal with reason selector.
- Scanner app (mobile): full-screen QR scanner, immediate colour-coded result (green/red/amber for suspended), zone list, offline CRL cache.
- Public organisation list: filterable table, no individual names, downloadable CSV.
- All voter/applicant screens WCAG 2.2 AA; EN + SW toggle.

---

## 10. Compliance Mapping

| Requirement | Provision | Implementation |
|---|---|---|
| Accreditation authority | Elections Act s.21A | IEBC as sole accreditation body |
| Media code of conduct | Media Council Act 2013 s.29 | Digital acknowledgement gate before badge issuance |
| DPA 2019 lawful basis | DPA 2019 s.30(1)(b) | Legal obligation; DPIA filed |
| Data retention | ECOWAS / IEBC policy | 7-year post-election retention; NPS data not retained |
| Audit log integrity | IEBC Regulations | Merkle-anchored append-only log |

---

## 11. Success Metrics + SLA

| Metric | Target |
|---|---|
| Application processing time (vetting complete) | <= 48 hours of submission |
| Badge QR verification response | <= 300 ms |
| CRL propagation delay | <= 5 minutes |
| Revocation to scanner reflection | <= next CRL sync cycle (max 5 min) |
| System availability during election day | >= 99.9% |
| Accredited media complaints re: access | 0 legitimate access denials due to system error |

---

## 12. Test Strategy

- Unit: badge signature generation + CRL signing logic.
- Integration: MCK API mock, NPS mock, M16 network-access list push.
- Load: 2,000 concurrent applicants submitting during window open.
- Penetration: badge serial enumeration; CRL tampering; revocation bypass.
- UAT: IEBC Media Officers + MCK representatives; journalist pilot cohort.
- Offline scanner test: CRL sync after deliberate network drop.

---

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date |
|---|---|
| MCK API integration agreed | T-270 (2026-11-13) |
| Portal MVP deployed to UAT | T-180 (2027-02-11) |
| IEBC Media Officer training | T-90 (2027-05-12) |
| Accreditation window opens | T-60 (2027-06-11) |
| Badge issuance begins | T-45 (2027-06-26) |
| Election day scanner deployment | T-0 (2027-08-10) |

---

## 14. Dependencies

- P05 Merkle audit log operational.
- P06 HSM provisioned with IEBC signing key for badges.
- M16 Election Command Centre network-access API available.
- MCK API MoU signed and sandbox access granted (T-270).

---

## 15. Risks + Mitigations (Top 5)

| Risk | Likelihood | Mitigation |
|---|---|---|
| MCK API unavailable during vetting | Medium | Fallback: manual MCK membership verification with uploaded certificate |
| Badge counterfeiting | Low | HSM-signed QR; CRL online check at each scan |
| CRL not synced to offline scanners | Medium | Scanners alert if CRL age > 30 min; manual block list distributed via SMS to supervisors |
| Surge applications at window open | High | Auto-scaling portal; queue-based processing; bulk org upload pathway reduces individual load |
| Revocation contested as arbitrary | Medium | Reason codes tied to specific regulation clauses; second-officer confirmation for non-emergency revocations |

---

## 16. Cross-References

- `M16-election-command-centre.md` — network-access list for live-feed permits
- `M43-observer-accreditation-portal.md` — shared vetting infrastructure
- `P05-merkle-audit.md` — immutable audit log
- `P06-hsm.md` — badge signing key
- `M11-public-portal.md` — public organisation list publication

---

## 17. Strategy Traceability

| Strategy Section | Requirement | Module Response |
|---|---|---|
| §XVIII Stakeholder Engagement | Media accreditation framework | FR-M42-001 to FR-M42-014 |
| §XVIII.4 Transparency | Public accredited-organisation list | FR-M42-013 |
| Elections Act s.21A | IEBC sole accreditation authority | Application lifecycle FR-M42-004 |
| Media Council Act 2013 s.29 | Code-of-conduct enforcement | FR-M42-008, FR-M42-011 |

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
