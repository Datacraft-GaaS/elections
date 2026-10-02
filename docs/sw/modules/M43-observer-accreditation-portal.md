# M43 — Observer Accreditation Portal

**Module ID:** M43
**Name:** Observer Accreditation Portal
**Runs on:** Central servers (Kenyan cloud) + the Commission back-office
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4

---

## 1. Purpose

Upstream accreditation of domestic and international election observers. Distinct from M15 (IAB Dashboard — ongoing audit role) and M34 (Observer App — field data collection tool). This module gates entry: an observer organisation applies, is vetted, and receives credentials before any field activity. Covers domestic NGOs (KEOG, ELOG), international missions (EU EOM, AU, Carter Center, IFES, Commonwealth), and individual observers under accredited organisations.

Strategy reference: §XVIII (Stakeholder Engagement). Elections Act s.21A; the Commission Observer Accreditation Regulations 2022.

---

## 2. User Personas

- **Observer Organisation Admin** — submits organisation accreditation; uploads MOU / legal documents; manages observer roster
- **Individual Observer** — applies under an accredited organisation; downloads credential; deploys to assigned area
- **the Commission Accreditation Officer** — reviews organisation and individual applications; runs background checks; issues or rejects
- **the Commission Director of Legal** — approves international-mission MOUs; signs gazette notice
- **M15 IAB Secretariat** — receives accredited-observer list for IAB parallel reporting
- **Field Deployment Coordinator** — assigns accredited observers to constituencies; tracks field presence
- **Results Centre Manager** — verifies observer credentials at entry

---

## 3. Functional Requirements

- **FR-M43-001** Organisation application: legal name, jurisdiction, registration documents, mandate letter, contact focal point, total observer count, geographic deployment plan, prior Kenya election history.
- **FR-M43-002** Organisation type classification: DOMESTIC_NGO / INTERNATIONAL_MISSION / DIPLOMATIC_MISSION / RELIGIOUS_BODY / ACADEMIC. Classification drives vetting depth and MOU requirement.
- **FR-M43-003** MOU workflow for INTERNATIONAL_MISSION organisations: draft MOU generated from template; the Commission Director of Legal signs via HSM; counterpart uploads signed copy; status advances to MOU_SIGNED before individual observers can apply.
- **FR-M43-004** Individual observer application under approved organisation: biographic details, NIN/Passport, photo (ICAO), language competencies, prior observer experience, deployment preference (region/county).
- **FR-M43-005** Background check: NPS clearance advisory; the Commission internal watch-list; prior accreditation revocation history; criminal record self-declaration.
- **FR-M43-006** Application lifecycle: SUBMITTED → DOCS_VERIFIED → BACKGROUND_CHECK → APPROVED / REJECTED → CREDENTIAL_ISSUED → DEPLOYED / SUSPENDED / REVOKED.
- **FR-M43-007** Credential issuance: digitally signed PDF credential with observer name, photo, organisation, deployment county, validity window, credential serial (UUID7); signed via P06 HSM; QR verifiable offline.
- **FR-M43-008** Field-deployment assignment: the Commission Coordinator assigns approved observers to specific constituencies; observer receives deployment letter with polling-station list; deployment data published to M34 Observer App (observer sees assigned stations).
- **FR-M43-009** Observer quota management: per-station observer quotas enforced (Elections Act s.21A); system prevents assignment beyond quota.
- **FR-M43-010** Observer organisation onboarding for known missions: pre-loaded profiles for KEOG, ELOG, EU EOM, AU, Carter Center, IFES, Commonwealth — org admin claims profile with document verification.
- **FR-M43-011** Revocation workflow: the Commission Accreditation Officer initiates with reason; credential serial added to CRL; field coordinator notified; M34 app credentials for that observer invalidated.
- **FR-M43-012** Accreditation window enforcement per the Commission gazette notice; late applications rejected.
- **FR-M43-013** Public summary: the Commission publishes count of accredited observers per organisation and per county; no individual names without consent.
- **FR-M43-014** Code-of-conduct acceptance: observer signs the Commission observer code digitally before credential issuance.
- **FR-M43-015** All mutations audit-logged; appended to P05 Merkle chain.

---

## 4. Non-Functional Requirements

- **NFR-M43-001** Portal p99 <= 1 s under 5,000 concurrent users (large international missions submit bulk rosters).
- **NFR-M43-002** Credential QR verification <= 300 ms.
- **NFR-M43-003** CRL refresh every 5 minutes; pushed to CDN.
- **NFR-M43-004** WCAG 2.2 AA; EN + SW; French and Arabic display options for international observers (display only, records in EN).
- **NFR-M43-005** DPA 2019 s.30(1)(b) lawful basis; biometric photo retained 7 years; NPS data not retained post-vetting; DPIA registered.
- **NFR-M43-006** Availability >= 99.9% during accreditation window and election day.
- **NFR-M43-007** Bulk roster import: CSV/XLSX up to 5,000 rows processed within 10 minutes.
- **NFR-M43-008** RBAC: Accreditation Officer cannot self-approve application where conflict of interest declared.

---

- **NFR-43-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-43` (replace 43 with the module number); alert ruleset ID `iems-43-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the ELOG Kenya programme director, I submit our organisation's accreditation application with our NGO registration and mandate, and receive approval within 5 working days.
2. As an EU EOM observer, I apply under the EU EOM accredited organisation using my passport, and download a credential QR within 48 hours of approval.
3. As a Commission Accreditation Officer, I see a vetting dashboard with NPS advisory, watch-list status, and prior revocation history before approving any individual.
4. As the Commission Director of Legal, I receive an auto-generated MOU draft for each international mission and sign it with my HSM token; the counterpart upload unlocks observer applications.
5. As a Field Deployment Coordinator, I assign 200 observers to 200 constituencies in a batch CSV upload, and each observer receives a deployment letter automatically.
6. As a Results Centre Manager, I scan an observer's QR and see their organisation, deployment county, and validity in under one second.
7. As a Commission Accreditation Officer, I revoke an observer's credential on a misconduct report; within 5 minutes all entry scanners reflect the revocation.
8. As a Carter Center observer, I can view my assigned polling stations in the M34 Observer App after my credential is issued.
9. As a member of the public, I can see how many observers each organisation deployed per county, without accessing individual names.
10. As an observer organisation admin, I upload a 3,000-row roster and track per-person approval status on a bulk dashboard.
11. As a Commission Commissioner, I receive a pre-election report showing accreditation rates per observer organisation and geographic coverage gaps.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import date, datetime
from enum import Enum

class OrgType(str, Enum):
    DOMESTIC_NGO = "DOMESTIC_NGO"
    INTERNATIONAL_MISSION = "INTERNATIONAL_MISSION"
    DIPLOMATIC_MISSION = "DIPLOMATIC_MISSION"
    RELIGIOUS_BODY = "RELIGIOUS_BODY"
    ACADEMIC = "ACADEMIC"

class ObserverStatus(str, Enum):
    SUBMITTED = "SUBMITTED"
    DOCS_VERIFIED = "DOCS_VERIFIED"
    BACKGROUND_CHECK = "BACKGROUND_CHECK"
    APPROVED = "APPROVED"
    REJECTED = "REJECTED"
    CREDENTIAL_ISSUED = "CREDENTIAL_ISSUED"
    DEPLOYED = "DEPLOYED"
    SUSPENDED = "SUSPENDED"
    REVOKED = "REVOKED"

class ObserverOrganisation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    name: str
    org_type: OrgType
    jurisdiction: str
    mou_required: bool
    mou_signed_at: datetime | None
    approved_at: datetime | None
    observer_quota: int
    created_at: datetime

class IndividualObserver(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    organisation_id: str
    full_name: str
    id_number: str
    nationality: str
    photo_url: str
    deployment_county_codes: list[str]
    assigned_constituency_codes: list[str]
    status: ObserverStatus
    credential_serial: str | None
    credential_issued_at: datetime | None
    valid_from: date
    valid_to: date
    revocation_reason: str | None
    created_at: datetime
    updated_at: datetime
```

---

## 7. API (OpenAPI 3.1 excerpt)

```yaml
openapi: "3.1.0"
info:
  title: M43 Observer Accreditation Portal API
  version: "1.0.0"
paths:
  /api/v1/organisations:
    post:
      summary: Submit organisation accreditation
      responses:
        "202": { description: "Application accepted" }
  /api/v1/organisations/{org_id}/observers:
    post:
      summary: Submit individual observer application
      responses:
        "202": { description: "Application queued" }
  /api/v1/observers/{id}/deploy:
    post:
      summary: Assign observer to constituencies
      security: [{ BearerAuth: [field_coordinator] }]
      responses:
        "200": { description: "Assignment confirmed" }
  /api/v1/credentials/{serial}/verify:
    get:
      summary: Verify observer credential (offline-capable)
      responses:
        "200": { description: "Credential valid" }
        "410": { description: "Credential revoked" }
  /api/v1/observers/{id}/revoke:
    post:
      summary: Revoke individual credential
      security: [{ BearerAuth: [accreditation_officer] }]
      responses:
        "200": { description: "Revocation recorded, CRL queued" }
  /api/v1/crl:
    get:
      summary: Revocation list for offline scanners
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
| NPS API | Outbound | Advisory background check |
| M15 IAB Dashboard | Outbound | Publish accredited-observer list for parallel reporting |
| M34 Observer App | Outbound | Push deployment assignments + credential validity |
| P05 Merkle / Audit Log | Outbound | Immutable mutation log |
| P06 HSM | Outbound | Credential signing; MOU signing |
| M42 Media Accreditation | Internal | Shared CRL infrastructure, vetting utilities |
| M16 Election Command Centre | Outbound | Field-presence dashboard |

---

## 9. UI/UX

- Organisation portal: multi-step wizard with document upload; MOU status tracker; roster bulk-upload with per-row error report.
- Observer self-application: 4-step form; photo upload with ICAO guideline tooltip; language-preference selector.
- the Commission vetting queue: card-based view with status chips; checklist side panel; approve/reject modal with statutory reason codes.
- Deployment board: map view of assigned observers per county with quota fill bars.
- Scanner app: shared with M42; QR scan returns observer-specific credential type indicator.
- All applicant screens WCAG 2.2 AA; EN + SW; French + Arabic optional display.

---

## 10. Compliance Mapping

| Requirement | Provision | Implementation |
|---|---|---|
| Accreditation authority | Elections Act s.21A | Commission-controlled approval workflow |
| Observer code of conduct | the Commission Regulations 2022 | Digital acknowledgement gate FR-M43-014 |
| Per-station observer quota | Elections Act s.21A | Quota enforcement FR-M43-009 |
| DPA 2019 lawful basis | s.30(1)(b) legal obligation | DPIA filed; data minimisation on NPS data |
| International mission MOU | the Commission Regulations | MOU workflow FR-M43-003 |

---

## 11. Success Metrics + SLA

| Metric | Target |
|---|---|
| Organisation application processing | <= 5 working days |
| Individual credential issuance (post-approval) | <= 48 hours |
| Credential QR verification | <= 300 ms |
| CRL propagation | <= 5 minutes |
| Geographic coverage (observers per constituency) | >= 85% constituencies with at least one observer |
| System availability election day | >= 99.9% |

---

## 12. Test Strategy

- Unit: MOU template generation; credential signature; CRL generation.
- Integration: NPS mock; M34 push API; M15 list sync.
- Load: 5,000 concurrent roster uploads at window open.
- UAT: KEOG, ELOG, EU EOM pilot cohorts; the Commission Accreditation Officers.
- Security: credential serial enumeration; revocation bypass; roster injection.

---

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date |
|---|---|
| Observer regulation alignment workshop | T-300 (2026-10-14) |
| MOU template approved by the Commission Legal | T-270 (2026-11-13) |
| Portal UAT with KEOG/ELOG | T-180 (2027-02-11) |
| International mission outreach + MOU signing | T-150 (2027-03-13) |
| Accreditation window opens | T-90 (2027-05-12) |
| Credential issuance begins | T-60 (2027-06-11) |
| Deployment assignments published to M34 | T-30 (2027-07-11) |

---

## 14. Dependencies

- P05 Merkle log, P06 HSM operational.
- M34 Observer App credential-validation API ready (T-60).
- M15 IAB Dashboard list-receive endpoint ready (T-45).
- NPS API MoU signed (T-270).
- the Commission Observer Accreditation Regulations 2022 confirmed unchanged for 2027 cycle.

---

## 15. Risks + Mitigations (Top 5)

| Risk | Likelihood | Mitigation |
|---|---|---|
| International missions submit late rosters | High | Pre-loaded org profiles; dedicated liaison officer per major mission from T-150 |
| NPS API unavailable during vetting | Medium | Advisory-only; manual clearance with uploaded self-declaration |
| Observer quota contention in high-interest constituencies | Medium | System enforces quota; overflow applicants offered alternate constituency |
| MOU negotiation delays international observers | Medium | Parallel-track: org approved conditionally, individuals queued pending MOU completion |
| Credential counterfeiting | Low | HSM-signed QR; online revocation check at every scan |

---

## 16. Cross-References

- `M15-iab-dashboard.md` — receives accredited-observer list
- `M34-observer-app.md` — receives deployment assignments and credential validity
- `M42-media-accreditation-portal.md` — shared CRL and scanning infrastructure
- `P05-merkle-audit.md` — audit chain
- `P06-hsm.md` — credential + MOU signing
- `M16-election-command-centre.md` — field presence visibility

---

## 17. Strategy Traceability

| Strategy Section | Requirement | Module Response |
|---|---|---|
| §XVIII Stakeholder Engagement | Observer accreditation framework | FR-M43-001 to FR-M43-015 |
| §XVIII.3 International Observers | MOU requirement for international missions | FR-M43-003 |
| Elections Act s.21A | the Commission sole accreditation authority; per-station quota | FR-M43-001, FR-M43-009 |
| the Commission Accreditation Regulations 2022 | Code of conduct; revocation | FR-M43-011, FR-M43-014 |

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
