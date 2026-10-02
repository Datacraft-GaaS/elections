# M46 — Party Primaries Oversight

**Module ID:** M46
**Name:** Party Primaries Oversight
**Runs on:** Central servers (Kenyan cloud) + party-facing portal + the Commission oversight back-office
**Owner:** Datacraft (pro-bono offering; outside the Commission tender scope)
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4; pro-bono module; Datacraft retains right to productise separately

---

## 1. Purpose

Party primaries are the single highest-fraud-risk event in the Kenyan electoral cycle (Strategy §XXI.A.1). the Commission has no constitutional mandate to run primaries but has an oversight function to receive and publish results and adjudicate complaints. This module provides: a reduced-feature KIEMS-variant for parties that opt in; a primary-specific integrity dashboard for the Commission oversight; primary-result publication; and primary-dispute intake routed to PPDT and the Political Parties Liaison Committee. Pro-bono offering distinct from the Commission tender scope.

Strategy reference: §XXI.A.1 (Party Primaries). Political Parties Act 2011 s.22; Elections Act s.33.

---

## 2. User Personas

- **Party Returning Officer** — operates the opt-in KIEMS-lite kit; submits primary results per ward/constituency
- **Party Primary Observer** — independent observer admitted under party rules; views integrity dashboard
- **the Commission Oversight Officer** — monitors primary-result publication; receives complaint flags; issues oversight notices
- **PPDT Clerk** — receives primary-dispute referrals
- **Candidate (Primary Contestant)** — views own primary results; submits disputes; tracks dispute status
- **Public User** — reads published primary results; no login required
- **M45 Party Registration** — system-to-system: party must be registered and ACTIVE to access this module

---

## 3. Functional Requirements

### KIEMS-Lite Opt-In

- **FR-M46-001** Party opt-in: registered party (ACTIVE status in M45) applies to use KIEMS-lite; the Commission Oversight Officer approves; party signs MoU committing to result publication and the Commission oversight.
- **FR-M46-002** KIEMS-lite kit configuration: subset of M05 BVI functionality — voter-roll import (party-supplied, not CVR), biometric or PIN voter authentication (party choice), ballot casting, result tallying. No ABIS integration. No national network connectivity required. Offline-capable.
- **FR-M46-003** Voter roll: party supplies voter roll per constituency (CSV; party members eligible to vote in that primary); system validates format but does not cross-reference against CVR (party-member data outside the Commission mandate).
- **FR-M46-004** Kit provisioning per constituency: kit serial bound to constituency code and primary election date; cannot be used at another constituency.
- **FR-M46-005** Result submission: party Returning Officer submits constituency primary results (candidate names, votes received, spoilt ballots, total turnout) via signed JSON upload to M46 backend; signature verified against party-provisioned certificate.

### Primary Result Publication

- **FR-M46-006** Results published to public portal within 4 hours of submission; per-constituency breakdown; downloadable CSV/JSON.
- **FR-M46-007** Results integrity: each result submission hash-anchored in P05 Merkle chain; public Merkle path verifiable.
- **FR-M46-008** Result amendment: party may request amendment within 48 hours of publication with supporting documentation; the Commission Oversight Officer approves amendment; original and amended versions both retained.

### Primary Integrity Dashboard

- **FR-M46-009** the Commission Oversight Officer dashboard: map view of primary result submission status per constituency (SUBMITTED / PENDING / LATE / DISPUTED); turnout anomaly flags (> 2 SD deviation from party's historical primary turnout where available).
- **FR-M46-010** Complaint density heat map: complaints per constituency displayed as choropleth; hot-spots highlighted for the Commission Oversight Officer attention.
- **FR-M46-011** the Commission oversight notice: Oversight Officer can issue a formal oversight notice to a party regarding specific constituency irregularities; notice published on the public portal alongside those constituency results.

### Primary Dispute Intake

- **FR-M46-012** Candidate dispute submission: losing candidate submits dispute form (constituency, grounds, supporting evidence) within 48 hours of result publication.
- **FR-M46-013** Party internal dispute first step: system requires attestation that candidate has first referred dispute to party internal dispute mechanism (PDM); if PDM unresolved within 7 days, dispute auto-escalates to PPDT intake.
- **FR-M46-014** PPDT referral: structured referral package (result PDF, dispute form, PDM response) generated and sent to PPDT Clerk; case number issued within 24 hours.
- **FR-M46-015** Dispute status tracker: candidates and parties track dispute lifecycle (SUBMITTED → PDM_REVIEW → PPDT_REFERRED → HEARING → DETERMINATION / WITHDRAWN).
- **FR-M46-016** Determination-driven result correction: if PPDT determination mandates result correction, Oversight Officer updates published result with determination reference; original result archived.

---

## 4. Non-Functional Requirements

- **NFR-M46-001** KIEMS-lite kit operates fully offline; result upload batch on connectivity restoration.
- **NFR-M46-002** Result publication within 4 hours of submission; p99 upload response <= 2 s.
- **NFR-M46-003** Public results portal p99 <= 2 s under 20,000 concurrent users (primary results draw significant public interest).
- **NFR-M46-004** WCAG 2.2 AA for public portal and candidate portal; EN + SW.
- **NFR-M46-005** DPA 2019: party-member voter rolls not stored beyond tallying session; only aggregate results retained by the Commission; DPIA registered.
- **NFR-M46-006** Merkle-anchored result storage; tamper detection on any stored result hash.
- **NFR-M46-007** Availability >= 99.5% during primary result submission windows (typically 2 weeks around primary election dates, T-90 to T-75).
- **NFR-M46-008** PPDT referral dispatch within 24 hours of PDM escalation trigger.

---

- **NFR-46-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-46` (replace 46 with the module number); alert ruleset ID `iems-46-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a party Secretary-General, I opt our party into KIEMS-lite oversight, sign the MoU online, and receive provisioned kits 30 days before our primary election.
2. As a party Returning Officer, I upload the constituency primary results after polls close and receive a Merkle receipt within 2 minutes.
3. As a member of the public, I can view primary results per constituency within 4 hours of result submission and download the full results CSV.
4. As a Commission Oversight Officer, I see a map of which constituencies have submitted results and which are late, with turnout anomaly flags highlighted.
5. As a losing candidate, I submit a primary dispute within 48 hours of result publication and receive a case reference number immediately.
6. As a candidate, after 7 days without a party PDM resolution, my dispute is automatically elevated to PPDT with a structured case file; I do not need to re-submit documents.
7. As the PPDT Clerk, I receive a pre-assembled referral package with the result PDF, dispute form, and PDM chronology, so I can open a case in 15 minutes.
8. As a Commission Oversight Officer, I issue a formal oversight notice to a party about irregularities in Mombasa County primary; the notice appears on the public portal next to those results within 1 hour.
9. As a journalist, I can verify that the published result JSON matches the Merkle hash published on the portal.
10. As a candidate, if PPDT issues a determination correcting my result, I can see the updated published result with the determination reference linked.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import date, datetime
from enum import Enum

class PrimaryOptIn(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    party_id: str                      # FK to M45 PoliticalParty
    election_cycle: str                # e.g. "2027-GENERAL"
    mou_signed_at: datetime
    approved_by: str
    approved_at: datetime

class PrimaryResult(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    party_id: str
    constituency_code: str
    position: str
    submitted_at: datetime
    submitted_by: str
    candidate_tallies: list[dict]      # [{candidate_name, votes}]
    spoilt_ballots: int
    total_turnout: int
    result_hash: str                   # SHA-256 of canonical JSON
    merkle_leaf_id: str | None
    published_at: datetime | None
    amended: bool
    amendment_reason: str | None

class PrimaryDispute(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    result_id: str
    complainant_name: str
    complainant_id: str
    grounds: str
    evidence_urls: list[str]
    status: str
    pdm_submitted_at: datetime | None
    ppdt_referral_at: datetime | None
    ppdt_case_number: str | None
    determination_url: str | None
    created_at: datetime
    updated_at: datetime
```

---

## 7. API (OpenAPI 3.1 excerpt)

```yaml
openapi: "3.1.0"
info:
  title: M46 Party Primaries Oversight API
  version: "1.0.0"
paths:
  /api/v1/opt-in:
    post:
      summary: Party opts into KIEMS-lite oversight
      security: [{ BearerAuth: [party_official] }]
      responses:
        "202": { description: "Opt-in application submitted" }
  /api/v1/results:
    post:
      summary: Submit constituency primary result
      security: [{ BearerAuth: [party_returning_officer] }]
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/PrimaryResult'
      responses:
        "201": { description: "Result received; Merkle receipt in response" }
    get:
      summary: Public results listing
      responses:
        "200": { description: "Paginated primary results" }
  /api/v1/results/{id}:
    get:
      summary: Single constituency primary result with Merkle path
      responses:
        "200": { description: "PrimaryResult with merkle_path" }
  /api/v1/disputes:
    post:
      summary: Submit primary dispute
      responses:
        "201": { description: "Dispute created; PDM clock started" }
  /api/v1/disputes/{id}:
    get:
      summary: Dispute status
      responses:
        "200": { description: "PrimaryDispute with status timeline" }
  /api/v1/oversight/notices:
    post:
      summary: the Commission issues oversight notice
      security: [{ BearerAuth: [oversight_officer] }]
      responses:
        "201": { description: "Notice published" }
```

---

## 8. Integrations

| System | Direction | Purpose |
|---|---|---|
| M45 Party Registration | Inbound | Party ACTIVE status gate for opt-in |
| M12 CRMS | Outbound | Primary winners surfaced for nomination confirmation |
| PPDT | Outbound | Dispute referral package; determination ingest |
| P05 Merkle Log | Outbound | Result and notice hash anchoring |
| M44 Turnout Forecasting | Outbound | Primary turnout data for historical baseline |
| M16 Election Command Centre | Outbound | Primary oversight layer during primary period |

---

## 9. UI/UX

- Party portal: opt-in wizard; kit request form; result upload with drag-drop JSON/CSV; receipt confirmation page.
- the Commission oversight dashboard: constituency map with status chips; anomaly flag side panel; notice-composition modal.
- Public results portal: searchable by party + constituency + position; Merkle path verifier; CSV/JSON download; oversight notice banner where issued.
- Candidate dispute portal: 3-step dispute form; status timeline view; PPDT case tracker.
- All public screens WCAG 2.2 AA; EN + SW.

---

## 10. Compliance Mapping

| Requirement | Provision | Implementation |
|---|---|---|
| the Commission oversight of primaries | Elections Act s.33 | Oversight dashboard + notice capability |
| Party primary obligation to publish results | PPA 2011 s.22 | Result publication FR-M46-006 |
| Primary dispute to PPDT | PPA 2011 s.40 | Dispute intake + PPDT referral FR-M46-013 |
| DPA 2019 party-member data | DPA 2019 s.26 | Voter rolls not retained post-tally; DPIA filed |
| Merkle integrity | the Commission Audit Regulations | P05 hash anchoring for all result submissions |

---

## 11. Success Metrics + SLA

| Metric | Target |
|---|---|
| Result publication time after submission | <= 4 hours |
| Dispute case number issuance | <= 24 hours |
| PPDT referral dispatch | <= 24 hours of PDM escalation trigger |
| Opt-in party result submission rate | >= 80% of opted-in constituencies submit within 6 hours of close |
| System availability during primary result window | >= 99.5% |
| Public portal p99 | <= 2 s |

---

## 12. Test Strategy

- Unit: Merkle-receipt generation; result hash; PDM escalation timer.
- Integration: M45 party status gate; P05 hash anchor; PPDT referral API.
- Load: 200 result uploads in 2-hour window (typical primary close burst).
- UAT: the Commission Oversight Officers; PPDT clerks; pilot party officials.
- Offline KIEMS-lite kit: network-loss simulation; batch upload on reconnect.

---

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date |
|---|---|
| PPDT MoU for dispute referral API | T-300 (2026-10-14) |
| KIEMS-lite kit firmware build | T-210 (2027-01-12) |
| Party opt-in portal open | T-150 (2027-03-13) |
| KIEMS-lite kit provisioning | T-120 (2027-04-12) |
| Primary election window (typical) | T-90 to T-75 (2027-05-12 to 2027-05-27) |
| Primary result submission window | T-75 close + 6 hours |
| Dispute intake window closes | T-73 |

---

## 14. Dependencies

- M45 Party Registration ACTIVE status check API (T-150).
- P05 Merkle log operational.
- PPDT dispute-referral API MoU (T-300).
- M12 CRMS primary-winner ingestion endpoint (T-75).
- Political Parties Act 2011 s.22 obligations confirmed unchanged for 2027 cycle.

---

## 15. Risks + Mitigations (Top 5)

| Risk | Likelihood | Mitigation |
|---|---|---|
| Low party opt-in rate makes module underutilised | High | Outreach campaign; the Commission endorsement; results publication requirement applies to all parties regardless of opt-in |
| Parties submit fraudulent results even with KIEMS-lite | Medium | Result hash + Merkle anchoring + published receipt; observer presence from M43 encouraged |
| PPDT overwhelmed with primary disputes | High | Structured intake reduces manual work; PPDT capacity planning based on primary count |
| Kit loss or compromise | Medium | Kit serial binding; remote-wipe capability; inventory tracked in M20 |
| Party PDM stage used to delay legitimate disputes | High | 7-day hard PDM deadline enforced by system; auto-escalation non-overridable |

---

## 16. Cross-References

- `M45-party-registration-management.md` — party ACTIVE status gate
- `M12-crms.md` — primary winner feeds nomination workflow
- `M43-observer-accreditation-portal.md` — accredited observers may attend primaries
- `M44-turnout-forecasting-analytics.md` — primary turnout as historical data source
- `M50-by-election.md` — analogous reduced-cycle election workflow
- `P05-merkle-audit.md` — result integrity
- `M20-inventory-management.md` — KIEMS-lite kit tracking

---

## 17. Strategy Traceability

| Strategy Section | Requirement | Module Response |
|---|---|---|
| §XXI.A.1 Party Primaries (highest fraud risk) | Oversight infrastructure for primaries | FR-M46-001 to FR-M46-016 |
| §XXI.A.4 Dispute Resolution | PPDT intake for primary disputes | FR-M46-012 to FR-M46-016 |
| PPA 2011 s.22 | Party result publication obligation | FR-M46-006, FR-M46-007 |
| Elections Act s.33 | the Commission oversight function | FR-M46-009 to FR-M46-011 |

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
