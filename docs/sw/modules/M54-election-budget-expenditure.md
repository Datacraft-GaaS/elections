# M54 — Election Budget and Expenditure Portal

**Module ID:** M54
**Name:** Election Budget and Expenditure Portal
**Runs on:** Central servers (Kenyan cloud) + IEBC Finance back-office
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

IEBC-side vote-book execution portal for planning, committing, and reporting on election expenditure. Distinct from M22 Campaign Finance Transparency (which tracks candidate-side spending). Covers internal procurement workflow, PPADA 2015 and PPRA compliance beyond §22, quarterly Controller of Budget reporting, and the Auditor-General interface. Acts as IEBC's internal Financial Management System (FMS) overlay for the election budget line. Also cross-linked to M59 (if external audit module) for audit-readiness artefacts.

## 2. User Personas

- **IEBC Chief Executive Officer** — approves procurement above delegated authority threshold
- **IEBC Finance Officer** — manages vote-book; captures actual expenditure; prepares CoB quarterly reports
- **Procurement Officer** — initiates and tracks tenders; PPADA compliance checklists
- **Budget Controller** — monitors budget vs actual per cost centre; flags overruns
- **PPRA Portal** (external) — receives statutory procurement notices and contract awards
- **Controller of Budget (CoB)** (external) — quarterly and annual report recipient
- **Auditor-General (OAG)** (external) — receives audit-readiness package post-election
- **IEBC Commissioner (Chair)** — signs off on major procurement decisions and CoB reports
- **IEBC DPO** — oversight for any personal data in contractor records

## 3. Functional Requirements

- **FR-M54-001** Budget plan: create election budget by line (kits, printing, personnel, transport, ICT, venues, communication, contingency); lock approved budget version; track amendments with approval trail.
- **FR-M54-002** Vote-book execution: capture actual expenditure per line per period; match against approved budget; compute variance; flag lines exceeding approved allocation by > 5% (amber) or > 15% (red).
- **FR-M54-003** Procurement workflow: initiate procurement requisition per PPADA 2015; route for approval per delegated authority matrix; system enforces procurement thresholds (Open Tender, Restricted, Direct Procurement, Framework Agreement); generate procurement plan for PPRA publication.
- **FR-M54-004** PPRA integration: submit annual and quarterly procurement plans; submit contract award notices; receive acknowledgement; store PPRA reference numbers.
- **FR-M54-005** Contract register: per contract — supplier, category, amount, start/end date, PPRA notice reference, payment schedule, delivery milestones, insurance/bond details; milestones tracked against actual delivery.
- **FR-M54-006** Payment processing: raise payment voucher; route for Finance Officer + CEO approval per amount tier; interface to IFMIS (Government of Kenya IFMIS) for actual payment execution; record payment confirmation reference.
- **FR-M54-007** By-election cost tracking: for each by-election event (M50), generate a subsidiary cost centre; track and report actual vs planned costs; produce the post-by-election cost summary required by M50.
- **FR-M54-008** Quarterly CoB report: auto-generate CoB progress report per Controller of Budget Act format; include budget vs actual per vote head, procurement status, pending payments, and narrative; submit to CoB portal API or as signed PDF; record submission date.
- **FR-M54-009** Auditor-General audit package: at financial year end and post-election, compile audit-readiness package — vote-book summary, contract register, payment vouchers, procurement plans, PPRA notices; HSM-sign and transmit to OAG secure portal.
- **FR-M54-010** Asset register: all capital items procured (kits, servers, vehicles) registered with serial, value, and custodian; feeds M20 Inventory Management for kit tracking.
- **FR-M54-011** Budget amendments: any reallocation between lines requires CEO approval above threshold; each amendment recorded with reason and approval reference.
- **FR-M54-012** Financial dashboards: real-time budget vs actual charts by line; procurement pipeline status; payment backlog.

## 4. Non-Functional Requirements

- **NFR-M54-001** Vote-book computation (full budget vs actual reconciliation) <= 60 s on demand.
- **NFR-M54-002** CoB quarterly report generation <= 5 min; auto-triggered 5 business days before quarterly deadline.
- **NFR-M54-003** IFMIS integration: payment confirmation received and recorded <= 1 business day of payment execution.
- **NFR-M54-004** Availability: >= 99.5% (back-office tool; planned maintenance windows permitted outside business hours).
- **NFR-M54-005** WCAG 2.2 AA for all IEBC Finance Officer screens.
- **NFR-M54-006** Languages: EN primary (all financial and legal documents); SW for internal dashboards.
- **NFR-M54-007** DPA 2019 lawful basis: contractor personal data (directors, contacts) processed under s.30(1)(b) (legal obligation); PPADA record-keeping; retained 10 years per Public Finance Management Act.
- **NFR-M54-008** Audit trail: every financial record (budget line, voucher, payment, contract) has immutable append-only history; no soft-delete.
- **NFR-M54-009** Role-based access: segregation of duties — Finance Officer cannot approve their own payment vouchers; Procurement Officer cannot approve their own requisitions.

- **NFR-54-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-54` (replace 54 with the module number); alert ruleset ID `iems-54-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the IEBC Finance Officer, I see a real-time budget vs actual dashboard by cost line, so I can identify potential overruns before they occur.
2. As a Procurement Officer, I initiate a kit-purchase tender; the system checks the amount against PPADA thresholds and routes it to Open Tender automatically.
3. As the IEBC CEO, I receive a procurement approval request above my delegated authority threshold on my dashboard with all supporting documents attached.
4. As the IEBC Finance Officer, the quarterly CoB report is auto-generated 5 days before the deadline with all required data pre-filled; I review and submit with one click.
5. As the Auditor-General, I receive a digitally signed audit-readiness package at financial year end with all procurement plans, contract register, and payment vouchers in one structured archive.
6. As the Budget Controller, I receive an amber alert when the transport cost line reaches 85% of its approved allocation, with a breakdown of committed vs spent.
7. As an IEBC Finance Officer managing a by-election, I see a subsidiary cost centre for the event with its own budget vs actual view.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime
from decimal import Decimal

class ElectionBudget(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    election_event_id: str
    financial_year: str              # e.g. "2027/28"
    version: int
    status: Literal["DRAFT","APPROVED","LOCKED","AMENDED"]
    total_approved_kes: Decimal
    approved_by: str                 # CEO user ID
    approved_at: datetime | None
    created_at: datetime

class BudgetLine(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    budget_id: str
    vote_head: str                   # e.g. "21-ICT-KITS", "22-PRINTING"
    description: str
    approved_kes: Decimal
    committed_kes: Decimal           # sum of active contracts
    actual_kes: Decimal              # sum of paid vouchers
    variance_kes: Decimal            # approved - actual
    status: Literal["ON_TRACK","AMBER","RED"]

class ProcurementRequisition(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    budget_line_id: str
    description: str
    estimated_kes: Decimal
    procurement_method: Literal["OPEN_TENDER","RESTRICTED","DIRECT","FRAMEWORK"]
    ppada_basis: str                 # statutory justification for method
    ppra_ref: str | None
    status: Literal["DRAFT","APPROVED","ADVERTISED","EVALUATED","AWARDED","CONTRACT_SIGNED","CLOSED"]
    awarded_to: str | None          # supplier name
    awarded_kes: Decimal | None
    contract_start: date | None
    contract_end: date | None
    created_by: str
    created_at: datetime

class PaymentVoucher(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    contract_id: str
    invoice_ref: str
    amount_kes: Decimal
    payment_date: date | None
    ifmis_ref: str | None
    raised_by: str
    approved_by: str | None
    status: Literal["DRAFT","PENDING_APPROVAL","APPROVED","PAID","REJECTED"]
    created_at: datetime
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/finance/budgets:
    post:
      summary: Create election budget
      security: [{CEOJwt: []}]
  /v1/finance/budgets/{id}/lines:
    get:
      summary: Budget lines with variance
      security: [{FinanceOfficerJWT: []}]
  /v1/finance/procurements:
    post:
      summary: Initiate procurement requisition
      security: [{ProcurementOfficerJWT: []}]
  /v1/finance/procurements/{id}/approve:
    post:
      summary: Approve requisition (role-gated by amount tier)
      security: [{CEOJwt: []}]
  /v1/finance/contracts:
    get:
      summary: Contract register
      security: [{FinanceOfficerJWT: []}]
  /v1/finance/vouchers:
    post:
      summary: Raise payment voucher
      security: [{FinanceOfficerJWT: []}]
  /v1/finance/cob-report:
    post:
      summary: Generate and submit quarterly CoB report
      security: [{FinanceOfficerJWT: []}]
  /v1/finance/audit-package:
    post:
      summary: Compile and submit OAG audit package
      security: [{CEOJwt: []}]
  /v1/finance/dashboard:
    get:
      summary: Real-time budget vs actual summary
      security: [{FinanceOfficerJWT: []}]
```

## 8. Integrations

- **M20 Inventory Management** — capital assets (kits, vehicles) registered in asset register; M20 receives serial numbers
- **M50 By-Election Operations** — per-by-election subsidiary cost centres; cost summary output
- **M55 HR + Payroll** — temporary-staff payroll costs feed into personnel budget line actual
- **PPRA Portal** (external) — procurement plan and contract award submission
- **IFMIS** (external, GoK) — payment execution and confirmation
- **Controller of Budget Portal** (external) — quarterly report submission
- **OAG Secure Portal** (external) — audit package delivery
- **P06 HSM** — CEO and Commissioner signatures on audit package and CoB reports

## 9. UI/UX Key Screens

1. **Budget Dashboard** — donut chart of budget vs actual by cost head; amber/red line indicators; total committed vs available.
2. **Procurement Pipeline** — Kanban view per procurement stage; PPADA method badge; PPRA submission status.
3. **Contract Register** — searchable table; milestone tracker per contract; insurance/bond expiry alerts.
4. **Payment Voucher Workspace** — create, approve, track payment; IFMIS confirmation reference field.
5. **CoB Report Builder** — auto-populated tables; narrative text fields; Commissioner sign-off button; submission confirmation with timestamp.
6. **Audit Package Compiler** — checklist of required artefacts; green/red status per item; one-click compile and HSM-sign.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Procurement thresholds and methods | PPADA 2015 ss.22-30; PPADA Regulations 2020 |
| Procurement plan publication | PPADA 2015 §26; PPRA Regulations |
| Quarterly CoB reporting | Controller of Budget Act 2016 §9 |
| Auditor-General access | Public Audit Act 2015 §6 |
| Public Finance Management | PFM Act 2012; PFM Regulations 2015 |
| Record retention (financial) | PFM Act 2012 §12; 10-year retention |
| Contractor data | DPA 2019 s.30(1)(b) |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Budget vs actual reconciliation latency | <= 60 s | Dashboard refresh log |
| CoB quarterly report auto-generated before deadline | 5 business days prior | Report generation timestamp |
| PPRA submissions within statutory window | 100% | PPRA confirmation log |
| Procurement segregation-of-duties violations | 0 | Access control audit |
| Audit package delivered to OAG post-election | <= T+60 | OAG delivery confirmation |

## 12. Test Strategy

- **Unit:** PPADA threshold routing; variance computation; CoB report template population.
- **Integration:** IFMIS payment confirmation; PPRA plan submission; OAG audit package transmission.
- **E2E:** Full procurement lifecycle from requisition through contract award, payment, and CoB report.
- **Security:** Segregation of duties — confirm Finance Officer cannot approve own voucher; Procurement Officer cannot approve own requisition.
- **Accessibility:** WCAG 2.2 AA on Finance Officer screens.
- **Audit:** Confirm no soft-delete possible on any financial record.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| FMS integration design with GoK IFMIS | 2026-10-01 | Technical MoU with National Treasury |
| Budget module live | 2027-01-01 | IEBC Finance team begins election budget |
| PPRA integration tested | 2027-02-01 | Procurement plan submitted via system |
| First IFMIS payment via system | 2027-03-01 | Pilot payment |
| CoB Q1 report auto-generated | 2027-04-01 | First quarterly submission |
| Audit package framework ready | T+14 (2027-08-24) | OAG package compiled post-election |

## 14. Dependencies

- IFMIS (GoK; National Treasury MoU)
- PPRA Portal (external API)
- Controller of Budget Portal (external API)
- OAG Secure Portal (external API)
- M20 Inventory Management
- M50 By-Election Operations
- M55 HR + Payroll Temporary Staff
- P06 HSM

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| IFMIS API not available for integration | High (GoK system reliability) | Medium | Fallback: manual IFMIS entry with system recording payment reference entered by Finance Officer |
| PPRA portal API changes without notice | Medium | Medium | Versioned PPRA adapter; weekly health check; manual upload fallback |
| Budget overrun in major cost line (e.g. kit procurement) | Medium | High | Real-time variance alerts at 85% and 100%; automatic escalation to CEO; amendment workflow required before further commitments |
| Segregation of duties bypassed under deadline pressure | Low | High | Hard system controls; no override without Commissioner countersignature and audit log entry |
| Audit package incomplete at OAG submission | Low | High | Automated checklist with red blocking if any required document missing; 30-day pre-deadline compilation run |

## 16. Cross-References

- `M20-inventory-management.md` — capital asset register
- `M22-campaign-finance-transparency.md` — candidate-side spending (distinct from this module)
- `M50-by-election-operations.md` — per-by-election cost centre
- `M55-hr-payroll-temporary-staff.md` — personnel cost feed

## 17. Strategy Traceability

- PPADA 2015 — procurement compliance
- Controller of Budget Act 2016 §9 — quarterly reporting obligation
- Public Audit Act 2015 §6 — Auditor-General access
- PFM Act 2012 — public finance management framework

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
