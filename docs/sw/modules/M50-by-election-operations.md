# M50 — By-Election Operations

**Module ID:** M50
**Name:** By-Election Operations
**Runs on:** Central servers (Kenyan cloud) + reduced kit deployment
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Support the full lifecycle of by-elections — triggered by death, recall, court nullification, or vacancy — within the IEMS platform. By-elections are smaller in scope (single constituency or ward) but require the full compliance and auditability of a general election. Key distinctions: reduced kit deployment, per-by-election unit pricing, per-by-election SLA tiers, and rapid activation from dormant state. Strategy §XXI.B.17.

## 2. User Personas

- **the Commission Commissioner** — activates by-election; approves gazette notice (via M52)
- **Constituency Returning Officer** — runs the by-election for the affected seat
- **Field Logistics Officer** — deploys reduced kit set; manages abbreviated logistics
- **Candidate (by-election)** — registers via M12 CRMS; files ECFA disclosures via M22
- **Party Agent** — observes and tallies at polling stations
- **Voter (affected constituency)** — participates via biometric verification; same kit workflow as general election
- **the Commission DPO** — oversees DPA 2019 compliance for the by-election voter register subset

## 3. Functional Requirements

- **FR-M50-001** By-election record: the Commission admin creates by-election event — seat type (Presidential [vacancy], Gubernatorial, Senator, MP, CWR, MCA), constituency/ward code, trigger reason (DEATH, RECALL, NULLIFICATION, VACANCY), vacancy date, polling date.
- **FR-M50-002** Voter register subset extraction: extract the relevant register subset (constituency or ward) from the main VPRS (M04); freeze subset at nomination close; provide printed copy and digital backup per Elections Act.
- **FR-M50-003** Reduced kit deployment plan: system computes minimum kit requirement based on polling stations in affected constituency/ward; generates deployment schedule and courier manifest from M20 Inventory Management.
- **FR-M50-004** Candidate nomination: route nominations through M12 CRMS with by-election flag; enforce same statutory nomination requirements (deposit, signatures) as general election.
- **FR-M50-005** Abbreviated campaign finance: M22 ECFA obligations apply; ceiling tiers for by-elections are the same per-seat category; shorter disclosure window (weekly from nomination close).
- **FR-M50-006** Tally and publication: use standard M10 RTS pipeline; by-election results published to M11 Public Portal on a by-election-specific page with seat context.
- **FR-M50-007** Per-by-election SLA: the Commission can configure per-event RTO/RPO targets (same or tighter than general election defaults); SLA dashboard per active by-election.
- **FR-M50-008** Per-by-election unit costing: system records actual deployed kits, staff headcount (M55), transport costs, and prints a post-by-election cost report for Controller of Budget.
- **FR-M50-009** Rapid activation: by-election can be activated from dormant state to "polls open" in <= 21 days for MCA vacancies; system flags if timeline is legally tight and lists outstanding statutory steps.
- **FR-M50-010** Multi-concurrent by-elections: support up to 10 simultaneous by-elections in different constituencies without resource contention.
- **FR-M50-011** Result certification: hooks into M51 for RO declaration and gazette notice via M52.
- **FR-M50-012** Petition window: same 28-day window as general election; M52 tracks deadline.

## 4. Non-Functional Requirements

- **NFR-M50-001** By-election activation (create event to voter register subset ready) <= 4 hours system processing time.
- **NFR-M50-002** Kit deployment plan generated within 30 minutes of by-election creation.
- **NFR-M50-003** Tally pipeline: same p99 performance as M10 (p99 ingest-to-confirmation <= 30 s).
- **NFR-M50-004** Support 10 concurrent by-elections with no shared-resource bottleneck.
- **NFR-M50-005** WCAG 2.2 AA for all voter-facing by-election screens.
- **NFR-M50-006** Languages: EN + SW baseline.
- **NFR-M50-007** DPA 2019 lawful basis: voter register subset processed under s.30(1)(b); retained per the Commission data-retention schedule; not accessible to other constituency officers.
- **NFR-M50-008** Availability: >= 99.9% during active by-election period (nomination close to T+30).

- **NFR-50-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-50` (replace 50 with the module number); alert ruleset ID `iems-50-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the Commission Commissioner, I activate a by-election for Starehe Constituency within 2 hours of a court nullification judgment, so the statutory timeline starts immediately.
2. As a Field Logistics Officer, I receive a kit deployment manifest showing exactly which 23 kits go to which polling stations, generated automatically from the constituency's station list.
3. As a candidate in a by-election, I use the same CRMS nomination portal as a general-election candidate, so I don't need separate training.
4. As a Constituency Returning Officer, I tally Forms 34A through the standard RO module; the system knows this is a by-election and routes the result to the correct M11 page.
5. As a voter in the affected constituency, my experience is identical to a general election: biometric check, ballot, verification slip.
6. As the Commission Finance team, I pull a post-by-election cost report showing actual kit days, staff count, and transport for budget justification to the Controller of Budget.
7. As the Commission legal officer, I see a statutory timeline checklist when I create the by-election showing all required gazette notices and their deadlines.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime

class ByElection(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    seat_type: Literal["MP","SENATOR","GOVERNOR","CWR","MCA","PRESIDENTIAL"]
    constituency_code: str
    ward_code: str | None
    trigger_reason: Literal["DEATH","RECALL","NULLIFICATION","VACANCY"]
    vacancy_date: date
    nomination_close: date
    polling_date: date
    status: Literal["DRAFT","GAZETTED","NOMINATIONS_OPEN","CAMPAIGN","POLLS_OPEN","TALLYING","DECLARED","CERTIFIED","PETITION_WINDOW","CLOSED"]
    kits_deployed: int | None
    staff_headcount: int | None
    sla_rto_minutes: int             # default 30
    sla_rpo_minutes: int             # default 5
    gazette_notice_id: str | None    # FK to M52
    created_by: str
    created_at: datetime

class ByElectionCostReport(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    by_election_id: str
    kit_days: int
    staff_days: int
    transport_kes: float
    printing_kes: float
    venue_kes: float
    other_kes: float
    total_kes: float
    generated_at: datetime
    submitted_to_cob: bool
    cob_submission_date: datetime | None
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/by-elections:
    post:
      summary: Create by-election event
      security: [{CommissionAdminJWT: []}]
    get:
      summary: List by-elections (active and historical)
      security: [{CommissionStaffJWT: []}]
  /v1/by-elections/{id}:
    get:
      summary: Get by-election detail with statutory timeline
      security: [{CommissionStaffJWT: []}]
  /v1/by-elections/{id}/register-subset:
    post:
      summary: Extract voter register subset from VPRS
      security: [{CommissionAdminJWT: []}]
  /v1/by-elections/{id}/kit-plan:
    get:
      summary: Get kit deployment plan
      security: [{LogisticsOfficerJWT: []}]
  /v1/by-elections/{id}/cost-report:
    get:
      summary: Get post-by-election cost report
      security: [{FinanceOfficerJWT: []}]
  /v1/by-elections/{id}/sla-dashboard:
    get:
      summary: Per-by-election SLA metrics
      security: [{CommissionAdminJWT: []}]
```

## 8. Integrations

- **M04 VPRS** — voter register subset extraction; freeze at nomination close
- **M10 RTS Back-End** — tally pipeline (by-election tagged partition)
- **M11 Public Portal** — by-election results page
- **M12 CRMS** — candidate nomination with by-election flag
- **M20 Inventory Management** — kit deployment manifest and tracking
- **M22 Campaign Finance** — ECFA obligations with by-election ceiling tier
- **M51 Result Declaration** — RO declaration workflow
- **M52 Election Calendar + Gazette** — statutory notices and petition window
- **M55 HR + Payroll Temporary Staff** — by-election staff headcount and payroll

## 9. UI/UX Key Screens

1. **By-Election Creation Wizard** — seat type, trigger reason, dates; statutory timeline auto-populates with gazette notice deadlines.
2. **By-Election Dashboard** — active by-elections list; per-event status badge; days to polling countdown; outstanding actions list.
3. **Kit Deployment Plan** — constituency map with station markers; per-kit assignment table; courier manifest PDF export.
4. **By-Election Tally Monitor** — same as M16 but scoped to the single constituency; forms received vs expected counter.
5. **Post-By-Election Cost Report** — line-item breakdown; Controller of Budget submission button.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| By-election trigger and timeline | Elections Act 2011 §11, §25 |
| Nomination requirements | Elections Act 2011 §22-§24 |
| ECFA ceiling for by-elections | ECFA 2013 s.22 (same per-seat tier) |
| Gazette notice of by-election | Elections Act §11(3); Constitution Art 101(4) |
| Petition window | Elections Act §75; Constitution Art 87 |
| Voter register use | Elections Act §5; DPA 2019 s.30(1)(b) |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Activation to register-subset ready | <= 4 h system processing | Activation log |
| Kit plan generation time | <= 30 min | Plan creation timestamp |
| Post-by-election cost report | Delivered <= T+14 | Report generation log |
| Concurrent by-elections supported | 10 without degradation | Load test |
| Statutory timeline compliance | 100% of required notices on time | M52 compliance dashboard |

## 12. Test Strategy

- **Unit:** Voter register subset extraction correctness (boundary conditions: ward vs constituency scope); kit-count computation.
- **Integration:** End-to-end by-election from creation through M52 gazette trigger and M55 payroll activation.
- **E2E:** Simulate concurrent 3 by-elections; confirm no register cross-contamination.
- **Performance:** Activate 10 concurrent by-elections; measure register extraction time.
- **Accessibility:** WCAG 2.2 AA on by-election public results page.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| By-election module UAT | 2027-01-15 | the Commission Operations acceptance |
| Register subset pipeline tested | 2027-02-01 | Integration with M04 VPRS |
| First dry-run by-election | 2027-04-01 | Mock event in one constituency |
| Production ready (standby) | T-90 (2027-05-11) | Activated per vacancy |

## 14. Dependencies

- M04 VPRS (voter register)
- M10 RTS Back-End (tally)
- M12 CRMS (candidate nomination)
- M20 Inventory Management (kit logistics)
- M22 Campaign Finance
- M51 Result Declaration
- M52 Election Calendar + Gazette
- M55 HR + Payroll Temporary Staff

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Court orders by-election with < 21-day notice | Medium | High | Rapid-activation checklist pre-built; statutory minimum enforced with legal team alert |
| Register subset includes out-of-scope voters | Low | High | Constituency/ward boundary validation against M18 Boundary module before freeze |
| Kit shortage during concurrent by-elections | Medium | Medium | M20 inventory reservation system blocks kits per active by-election |
| ECFA filing non-compliance (short campaign) | High | Medium | Automated daily reminder from nomination close; non-compliance flag raised to the Commission officer at T+3 |
| Post-by-election cost report not submitted to CoB | Medium | Medium | Automated reminder to Finance Officer; M54 budget module tracks submission |

## 16. Cross-References

- `M04-vprs.md` — voter register source
- `M10-rts-backend.md` — tally pipeline
- `M12-crms.md` — candidate nomination
- `M20-inventory-management.md` — kit logistics
- `M22-campaign-finance-transparency.md` — ECFA obligations
- `M51-result-declaration-certification.md` — RO declaration
- `M52-election-calendar-gazette.md` — gazette notices
- `M54-election-budget-expenditure.md` — cost reporting
- `M55-hr-payroll-temporary-staff.md` — by-election staffing

## 17. Strategy Traceability

- Strategy §XXI.B.17 — By-Election Operations module
- Elections Act 2011 §11, §25 — by-election triggers and timeline
- ECFA 2013 s.22 — campaign finance ceiling
- Constitution Art 101(4) — vacancy and by-election gazette requirement

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
