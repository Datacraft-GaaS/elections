# M45 — Party Registration & Management

**Module ID:** M45
**Name:** Party Registration & Management
**Runs on:** Central servers (Kenyan cloud) + Registrar of Political Parties back-office
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4

---

## 1. Purpose

Automate the full lifecycle of political party registration and renewal under the Political Parties Act 2011 (PPA 2011) and the Registrar of Political Parties (RPP) mandate. Covers new party registration, annual returns, coalition registration, party symbol and colour metadata management (symbol image registry deferred to M67, party-facing metadata here), governance disclosure, and faction-dispute intake routed to PPDT. Integrates downstream with M12 CRMS (candidate party affiliation) and M47 (nomination fee).

Strategy reference: §XXI.A (Political Party Ecosystem). Political Parties Act 2011; Political Parties (Amendment) Act 2022; Elections Act s.22.

---

## 2. User Personas

- **Party Secretary-General** — submits registration or renewal application; uploads constitutive documents; manages party officials roster
- **Party Finance Officer** — files annual returns (audited accounts, membership fees, political financing disclosures)
- **Registrar of Political Parties** — reviews applications; approves, conditionally approves, or rejects; issues gazette notice
- **RPP Deputy Registrar** — handles routine renewals; flags disputes to Registrar
- **PPDT Clerk** — receives faction-dispute referrals from this module; manages case numbers
- **the Commission Commissioner** — views party compliance dashboard; approves coalition registrations requiring the Commission sign-off
- **M12 CRMS Integration** — system-to-system consumer of party registration status
- **Public User** — reads the public party register; no login required

---

## 3. Functional Requirements

### Party Registration

- **FR-M45-001** New party registration application: party name (EN + SW if applicable), abbreviation, ideology, founding document (constitution), party headquarters address, regional offices, party officials roster (officials + NIN), proposed symbol (image upload, stored as metadata FK to M67 Symbol Registry), proposed party colour (hex code), supporting membership list (minimum 1,000 registered voters across at least 24 counties per PPA 2011 s.7).
- **FR-M45-002** Membership list validation: cross-reference uploaded supporter list against M02 CVR (voter register); system returns count of verified registered voters per county; must meet PPA 2011 s.7 threshold before application advances.
- **FR-M45-003** Party name and symbol uniqueness check: name and abbreviation checked against active party register; symbol image checked against M67 Symbol Registry for similarity (perceptual hash threshold); block on conflict.
- **FR-M45-004** Application lifecycle: DRAFT → SUBMITTED → DOCS_REVIEW → MEMBERSHIP_VERIFIED → REGISTRAR_REVIEW → APPROVED / CONDITIONAL / REJECTED → GAZETTED.
- **FR-M45-005** Conditional approval: Registrar issues conditional approval with specific remediation items; party has 30 days to satisfy conditions or application lapses.
- **FR-M45-006** Annual renewal: registered parties submit annual return (audited financial accounts, updated officials roster, membership count update, governance compliance self-declaration) by 31 March each year; failure triggers SUSPENDED status after 30-day grace period.
- **FR-M45-007** Gazette publication: on approval, system generates gazette notice draft (PDF, the Commission standard format); Registrar signs via HSM; notice published to public register within 14 days of approval.
- **FR-M45-008** Party deregistration: Registrar initiates deregistration with statutory ground (PPA 2011 s.17); party notified with 21-day appeal window; deregistration confirmed after window; downstream systems notified (M12, M47).

### Coalition Registration

- **FR-M45-009** Coalition application: coalition name, member parties (each must be registered and in ACTIVE status), coalition agreement document, seat-allocation formula, dissolution trigger.
- **FR-M45-010** Coalition seat-allocation formula stored and surfaced in M12 CRMS nomination workflow.
- **FR-M45-011** Coalition dissolution: any member party may trigger dissolution; coalition status set to DISSOLVED; downstream systems notified.

### Faction Dispute Management

- **FR-M45-012** Faction-dispute intake: any party member may submit a dispute (rival leadership claim, expelled-member grievance, branch election dispute) via a structured form; dispute assigned a case number and routed to PPDT Clerk.
- **FR-M45-013** Dispute status tracker: parties and disputants can track case status (RECEIVED → UNDER_REVIEW → HEARING_SCHEDULED → DETERMINATION_ISSUED / WITHDRAWN).
- **FR-M45-014** PPDT determination ingest: PPDT uploads signed determination PDF; system updates party record if determination mandates a change (e.g., leadership reinstatement); change logged.

### Governance Disclosure

- **FR-M45-015** Political financing disclosure: parties file semi-annual disclosure of income sources per Political Parties (Amendment) Act 2022 s.24; foreign-origin funds flagged for RPP review.
- **FR-M45-016** Governance compliance score: system computes a compliance score (0-100) per party based on timeliness of annual return, dispute-free record, financing disclosures; score published on public register.

---

## 4. Non-Functional Requirements

- **NFR-M45-001** Portal p99 <= 1 s under 500 concurrent party users.
- **NFR-M45-002** Membership list CVR cross-reference: <= 30 minutes for a 10,000-row upload.
- **NFR-M45-003** WCAG 2.2 AA for applicant portal and public register; EN + SW.
- **NFR-M45-004** DPA 2019 s.30(1)(b) (legal obligation) for party official personal data; officials' NIN not exposed on public register; retained 7 years.
- **NFR-M45-005** Availability >= 99.5% during annual return filing window (January-March).
- **NFR-M45-006** All Registrar decisions audit-logged; appended to P05 Merkle chain.
- **NFR-M45-007** M12 CRMS sync: party registration status change propagated to M12 within 5 minutes.

---

- **NFR-45-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-45` (replace 45 with the module number); alert ruleset ID `iems-45-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a party Secretary-General, I submit our registration application online, upload our constitution and 1,200-member supporter list, and receive a CVR-verified supporter count within 30 minutes.
2. As the RPP Deputy Registrar, I see a vetting dashboard with membership-threshold pass/fail, symbol uniqueness result, and document checklist for each application.
3. As the Registrar, I issue a conditional approval with two specific remediation items, and the system sends the party a 30-day countdown notice automatically.
4. As a party official, I receive an email reminder 60 days before the annual return deadline and can file the return online with audited accounts upload.
5. As a party member with a branch election dispute, I submit a structured dispute form and receive a PPDT case number within 24 hours.
6. As the PPDT Clerk, I receive structured dispute referrals with all supporting documents pre-assembled, so I can open a case file without manual collection.
7. As a member of the public, I view the party register showing each party's compliance score, symbol, and registration date without logging in.
8. As the M12 CRMS system, I receive a webhook notification within 5 minutes when a party's registration status changes, so candidate nomination workflows reflect the current status.
9. As a coalition party Secretary-General, I submit a coalition agreement and both member parties confirm participation; the coalition appears in M12 as a valid nomination affiliation.
10. As the RPP Finance Officer, I flag a foreign-origin income item in a party's semi-annual disclosure and escalate it to the Registrar's review queue automatically.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import date, datetime
from enum import Enum

class PartyStatus(str, Enum):
    DRAFT = "DRAFT"
    SUBMITTED = "SUBMITTED"
    DOCS_REVIEW = "DOCS_REVIEW"
    MEMBERSHIP_VERIFIED = "MEMBERSHIP_VERIFIED"
    REGISTRAR_REVIEW = "REGISTRAR_REVIEW"
    APPROVED = "APPROVED"
    CONDITIONAL = "CONDITIONAL"
    REJECTED = "REJECTED"
    GAZETTED = "GAZETTED"
    SUSPENDED = "SUSPENDED"
    DEREGISTERED = "DEREGISTERED"

class PoliticalParty(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    name_en: str
    name_sw: str | None
    abbreviation: str
    ideology: str
    status: PartyStatus
    registration_number: str | None
    gazette_notice_url: str | None
    symbol_registry_id: str | None      # FK to M67 Symbol Registry
    colour_hex: str
    hq_address: str
    verified_membership_count: int
    compliance_score: int               # 0-100
    coalition_id: str | None
    created_at: datetime
    updated_at: datetime

class CoalitionRegistration(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    name: str
    member_party_ids: list[str]
    agreement_doc_url: str
    seat_allocation_formula: str
    status: str                         # ACTIVE / DISSOLVED
    dissolved_at: datetime | None
    created_at: datetime

class FactionDispute(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    party_id: str
    case_number: str
    dispute_type: str
    submitted_by: str
    description: str
    status: str
    ppdt_determination_url: str | None
    created_at: datetime
    updated_at: datetime
```

---

## 7. API (OpenAPI 3.1 excerpt)

```yaml
openapi: "3.1.0"
info:
  title: M45 Party Registration API
  version: "1.0.0"
paths:
  /api/v1/parties:
    post:
      summary: Submit new party registration application
      responses:
        "202": { description: "Application accepted" }
    get:
      summary: Public party register (paginated)
      responses:
        "200": { description: "List of registered parties" }
  /api/v1/parties/{id}/annual-return:
    post:
      summary: File annual return
      security: [{ BearerAuth: [party_official] }]
      responses:
        "202": { description: "Return filed" }
  /api/v1/parties/{id}/status:
    get:
      summary: Get party registration status
      responses:
        "200": { description: "PartyStatus with detail" }
  /api/v1/coalitions:
    post:
      summary: Register a coalition
      security: [{ BearerAuth: [party_official] }]
      responses:
        "202": { description: "Coalition registration pending member confirmation" }
  /api/v1/disputes:
    post:
      summary: Submit faction dispute
      responses:
        "201": { description: "Dispute created with PPDT case number" }
  /api/v1/parties/{id}/status:
    get:
      summary: M12 webhook — party status change event (SSE)
      responses:
        "200":
          content:
            text/event-stream:
              schema:
                type: object
                properties:
                  party_id: { type: string }
                  new_status: { type: string }
                  changed_at: { type: string, format: date-time }
```

---

## 8. Integrations

| System | Direction | Purpose |
|---|---|---|
| M02 BVR Backend | Outbound | Membership-list CVR cross-reference |
| M12 CRMS | Outbound | Party status webhook; nomination affiliation |
| M47 Nomination Fee | Outbound | Party status required before nomination fee accepted |
| M67 Symbol Registry | Outbound | Symbol uniqueness check; symbol image storage |
| PPDT | Outbound | Dispute referrals; determination ingest |
| P05 Merkle Log | Outbound | Decision audit trail |
| P06 HSM | Outbound | Gazette notice signing |

---

## 9. UI/UX

- Party portal: wizard for new registration (5 steps); document upload with format validator; membership CSV upload with row-level error report; annual return filing wizard.
- RPP back-office: application queue with status filter; membership-threshold meter; symbol uniqueness result panel; approve/conditional/reject modal.
- Public register: searchable table (name, abbreviation, status, compliance score, symbol thumbnail); compliance score badge (green/amber/red); gazette notice download.
- All screens WCAG 2.2 AA; EN + SW.

---

## 10. Compliance Mapping

| Requirement | Provision | Implementation |
|---|---|---|
| Party registration threshold | PPA 2011 s.7 | Automated CVR cross-reference FR-M45-002 |
| Annual return obligation | PPA 2011 s.14 | Annual return workflow FR-M45-006 |
| Deregistration grounds | PPA 2011 s.17 | Deregistration lifecycle FR-M45-008 |
| Political financing disclosure | PP (Amendment) Act 2022 s.24 | FR-M45-015 |
| Faction dispute tribunal | PPA 2011 s.40 | PPDT intake FR-M45-012 |
| DPA 2019 | s.30(1)(b) legal obligation | Officials' NIN protected; DPIA filed |

---

## 11. Success Metrics + SLA

| Metric | Target |
|---|---|
| Membership CVR cross-reference completion | <= 30 minutes for 10,000 rows |
| Registrar decision turnaround | <= 5 working days from docs complete |
| Annual return compliance rate (registered parties) | >= 90% file on time |
| M12 party status sync delay | <= 5 minutes |
| System availability (filing window) | >= 99.5% |
| Dispute case number issued | <= 24 hours of submission |

---

## 12. Test Strategy

- Unit: membership-threshold validation; symbol perceptual-hash comparison; compliance-score computation.
- Integration: M02 CVR mock; M12 webhook delivery; M67 uniqueness check; PPDT referral API.
- Load: 50 parties filing annual returns concurrently during March deadline.
- UAT: RPP staff; representative party officials from KEOG member parties; PPDT clerks.
- Security: party official access isolation; NIN exposure audit.

---

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date |
|---|---|
| RPP requirements workshop | T-365 (2026-08-10) |
| M02 membership cross-reference API agreed | T-300 (2026-10-14) |
| M67 Symbol Registry integration | T-270 (2026-11-13) |
| Portal MVP UAT with RPP | T-180 (2027-02-11) |
| Annual return filing for 2026-cycle test run | T-150 (2027-03-13) |
| Party register public launch | T-90 (2027-05-12) |
| M12 integration live | T-60 (2027-06-11) |

---

## 14. Dependencies

- M02 BVR backend CVR query API (T-300).
- M67 Symbol Registry perceptual-hash API (T-270).
- M12 CRMS webhook consumer ready (T-60).
- PPDT MoU for case-referral API (T-270).
- Political Parties Act 2011 and 2022 Amendment confirmed as operative legal framework.

---

## 15. Risks + Mitigations (Top 5)

| Risk | Likelihood | Mitigation |
|---|---|---|
| CVR extract outdated relative to registration date | Medium | Cross-reference uses registration-date-stamped CVR snapshot; timestamp logged |
| Symbol similarity threshold too strict, blocking legitimate parties | Medium | Configurable threshold; Registrar can override with reason |
| PPDT integration stalled by MoU delay | Medium | Fallback: email-based referral with structured PDF; API integrated post-MoU |
| Party delays annual return causing cascade to M12 CRMS | High | 60-day advance reminders; automated SUSPENDED flag propagated to M12 |
| Foreign financing disclosure resisted by parties | Medium | Non-disclosure triggers compliance-score penalty and RPP review flag |

---

## 16. Cross-References

- `M12-crms.md` — candidate party affiliation consumer
- `M47-nomination-fee-payment.md` — requires active party status
- `M67-symbol-registry.md` — symbol uniqueness + storage
- `M46-party-primaries-oversight.md` — party primary results linked to party record
- `P05-merkle-audit.md` — decision audit chain
- `P06-hsm.md` — gazette notice signing

---

## 17. Strategy Traceability

| Strategy Section | Requirement | Module Response |
|---|---|---|
| §XXI.A Political Party Ecosystem | Party registration management | FR-M45-001 to FR-M45-016 |
| §XXI.A.3 Coalition Framework | Coalition registration | FR-M45-009 to FR-M45-011 |
| §XXI.A.4 Dispute Resolution | PPDT integration | FR-M45-012 to FR-M45-014 |
| PPA 2011 s.7 | Membership threshold | FR-M45-001, FR-M45-002 |
| PP (Amendment) Act 2022 s.24 | Financing disclosure | FR-M45-015 |

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
