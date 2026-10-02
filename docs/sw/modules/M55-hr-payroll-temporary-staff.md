# M55 — HR and Payroll for Temporary Election Staff

**Module ID:** M55
**Name:** HR and Payroll for Temporary Election Staff
**Runs on:** Central servers (Kenyan cloud) + the Commission HR back-office + M-Pesa disbursement integration
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Manage recruitment, contracting, payroll, and statutory deductions for approximately 250,000 temporary election staff (Presiding Officers, Deputy Presiding Officers, Polling Clerks, Registration Clerks, Returning Officers, Support Staff). Distinct from M33 Personnel Dispatch (which handles field deployment logistics and task assignment). This module covers: recruitment portal, contract issuance, per-diem and allowance calculation, M-Pesa bulk payout, KRA PAYE withholding, NSSF contributions, and NHIF/SHIF deductions. Feeds from M38 LMS certification for eligibility gate.

## 2. User Personas

- **the Commission HR Officer** — manages recruitment campaigns; issues contracts; runs payroll
- **the Commission Finance Officer** — approves payroll disbursement runs; reconciles with M54 budget line
- **Temporary Staff Member (Applicant)** — applies online; receives contract; confirms acceptance; receives pay
- **M38 LMS** — provides certification status per staff member (eligibility gate for contract issuance)
- **KRA iTax** (external) — receives PAYE returns and payments
- **NSSF Portal** (external) — receives contribution returns and payments
- **NHIF/SHIF Portal** (external) — receives contribution returns and payments
- **M-Pesa Bulk API** (Safaricom) — disburses per-diem and allowances to staff phone numbers
- **the Commission DPO** — DPA 2019 DPIA oversight for staff personal and financial data
- **Auditor-General** — reviews payroll records post-election via M54 audit package

## 3. Functional Requirements

- **FR-M55-001** Recruitment portal: publish vacancy notices per role and county; online application form (name, national ID, phone, education qualifications, experience, county preference); application acknowledgement via SMS.
- **FR-M55-002** Eligibility screening: cross-check applicant national ID against M03 ABIS and M12 CRMS to exclude registered candidates and their close relatives (Elections Act §27 conflicts); flag disqualified applicants.
- **FR-M55-003** LMS certification gate: before contract issuance, verify applicant has completed and passed the required training module(s) in M38 LMS for their role type; block contract for uncertified applicants.
- **FR-M55-004** Shortlisting and selection: the Commission HR Officer scores applications per published criteria; system generates shortlist; notifications to shortlisted applicants.
- **FR-M55-005** Contract generation: auto-generate fixed-term employment contract (per role, county, election event, start/end dates, daily rate, statutory deductions schedule); PDF with the Commission digital signature; sent to applicant via email + SMS with acceptance link.
- **FR-M55-006** Contract acceptance: applicant accepts digitally (OTP-confirmed) or submits wet-ink scan; contract status set to ACCEPTED; feeds into payroll roster.
- **FR-M55-007** Per-diem and allowance calculation: compute gross pay per staff member = daily rate x days engaged + transport allowance + meal allowance per approved schedule; role-specific rates configurable by the Commission HR.
- **FR-M55-008** Statutory deductions: compute KRA PAYE per Kenya Revenue Authority tax bands for casual employment; NSSF Tier I + Tier II contributions per NSSF Act 2013 (amended 2023); NHIF/SHIF contribution per current rates; deductions itemised per staff member on payslip.
- **FR-M55-009** Payroll run: Finance Officer initiates payroll run for a given event-phase (e.g. "Polling Day + 2 days"); system computes net pay per staff member; generates bulk M-Pesa payment file; Finance Officer approves; system submits to Safaricom M-Pesa Bulk API; records payment reference per staff member; SMS confirmation to each payee.
- **FR-M55-010** Failed payment handling: M-Pesa failed payments (wrong number, SIM not registered) logged; the Commission HR Officer contacts staff for correction; re-run within 48 hours.
- **FR-M55-011** KRA PAYE return: after each payroll run, generate PAYE P9 return per staff member; aggregate monthly PAYE liability; submit to KRA iTax API; record payment reference.
- **FR-M55-012** NSSF and NHIF/SHIF returns: generate monthly contribution schedules; submit to respective portals; record confirmation.
- **FR-M55-013** Payslip: each staff member receives a PDF payslip (gross, deductions, net) via SMS link or email.
- **FR-M55-014** By-election and referendum staffing: for M49 and M50 events, activate subsidiary staffing campaigns scoped to affected constituencies; full payroll applies.
- **FR-M55-015** Post-election staff certificate: system generates a service certificate per temporary staff member on demand; used for future employment reference.

## 4. Non-Functional Requirements

- **NFR-M55-001** Payroll computation for 250,000 staff: full run completed within 4 hours.
- **NFR-M55-002** M-Pesa bulk disbursement: 250,000 payments submitted to Safaricom API within 6 hours of Finance Officer approval.
- **NFR-M55-003** Recruitment portal p99 response time <= 2 s under 50,000 concurrent applicants during vacancy-notice period.
- **NFR-M55-004** WCAG 2.2 AA for applicant-facing recruitment portal and payslip portal.
- **NFR-M55-005** Languages: EN + SW for all applicant-facing screens and payslips.
- **NFR-M55-006** DPA 2019 lawful basis: staff personal and financial data processed under s.30(1)(b) (legal obligation — employment contract and statutory deductions); DPIA registered with ODPC; staff data retained 7 years per KRA and PFM requirements; applicant data for non-selected candidates retained 6 months then deleted.
- **NFR-M55-007** Segregation of duties: HR Officer cannot approve their own payroll run; Finance Officer approval required.
- **NFR-M55-008** Audit trail: every payroll run, deduction, and payment immutably logged; payslip hash stored; cannot be modified post-approval.
- **NFR-M55-009** Availability: >= 99.5% during active election period (nomination close to T+30).

- **NFR-55-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-55` (replace 55 with the module number); alert ruleset ID `iems-55-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a temporary staff applicant, I apply online in 10 minutes, providing my national ID and selecting my preferred county; I receive an SMS confirming receipt.
2. As a Commission HR Officer, I see a dashboard showing LMS certification status per applicant pool for each role type, so I can identify under-certified groups before the shortlisting deadline.
3. As a shortlisted Presiding Officer, I receive a contract via email and accept it with an OTP confirmation; I do not need to visit the Commission office.
4. As a Commission Finance Officer, I approve the Polling Day payroll run; within 6 hours all 250,000 staff receive M-Pesa notifications confirming their payment.
5. As a Polling Clerk, I receive a PDF payslip via SMS link showing my gross pay, PAYE deduction, NSSF contribution, and net pay within 24 hours of the payroll run.
6. As the Commission HR Officer, I see a failed-payment list the morning after the payroll run and can re-run failed payments after correcting phone numbers.
7. As a Commission Legal Officer, I am alerted when an applicant's national ID matches a registered candidate's record, blocking their contract automatically.
8. As the KRA, I receive a complete PAYE P9 return for all 250,000 temporary staff via iTax API within 5 days of each payroll run.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime
from decimal import Decimal

class StaffApplication(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    election_event_id: str
    role_type: Literal["PRESIDING_OFFICER","DEPUTY_PO","POLLING_CLERK","REGISTRATION_CLERK","RETURNING_OFFICER","SUPPORT_STAFF"]
    applicant_national_id: str
    applicant_name: str
    phone_number: str                # M-Pesa registered
    county_preference: str
    lms_certified: bool
    lms_certificate_id: str | None   # FK to M38
    conflict_flag: bool              # Elections Act §27 conflict detected
    status: Literal["RECEIVED","SCREENED","SHORTLISTED","OFFERED","ACCEPTED","REJECTED","WITHDRAWN"]
    created_at: datetime

class EmploymentContract(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    application_id: str
    election_event_id: str
    role_type: str
    county_code: str
    constituency_code: str | None
    start_date: date
    end_date: date
    daily_rate_kes: Decimal
    transport_allowance_kes: Decimal
    meal_allowance_kes: Decimal
    contract_pdf_hash: str
    accepted_at: datetime | None
    acceptance_method: Literal["OTP","WET_INK_SCAN"] | None
    status: Literal["DRAFT","SENT","ACCEPTED","DECLINED","EXPIRED"]
    created_at: datetime

class PayrollRun(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    election_event_id: str
    phase: str                       # e.g. "POLLING_DAY_PLUS_2"
    staff_count: int
    gross_total_kes: Decimal
    paye_total_kes: Decimal
    nssf_total_kes: Decimal
    shif_total_kes: Decimal
    net_total_kes: Decimal
    approved_by: str                 # Finance Officer ID
    approved_at: datetime | None
    mpesa_batch_id: str | None
    mpesa_submitted_at: datetime | None
    failed_payments: int
    status: Literal["DRAFT","PENDING_APPROVAL","APPROVED","SUBMITTED","COMPLETE","PARTIAL_FAILURE"]
    created_at: datetime

class PayrollLine(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    payroll_run_id: str
    contract_id: str
    days_engaged: int
    gross_kes: Decimal
    paye_kes: Decimal
    nssf_kes: Decimal
    shif_kes: Decimal
    net_kes: Decimal
    mpesa_phone: str
    mpesa_ref: str | None
    payment_status: Literal["PENDING","PAID","FAILED","RERUN_PENDING"]
    paid_at: datetime | None
    payslip_hash: str | None
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/hr/applications:
    post:
      summary: Submit staff application
      security: [{PublicWithOTP: []}]
    get:
      summary: List applications (HR Officer)
      security: [{HROfficerJWT: []}]
  /v1/hr/applications/{id}/shortlist:
    post:
      summary: Mark application shortlisted
      security: [{HROfficerJWT: []}]
  /v1/hr/contracts:
    post:
      summary: Generate and send contract
      security: [{HROfficerJWT: []}]
  /v1/hr/contracts/{id}/accept:
    post:
      summary: Applicant accepts contract via OTP
      security: [{OTPAuth: []}]
  /v1/hr/payroll-runs:
    post:
      summary: Create payroll run for a phase
      security: [{HROfficerJWT: []}]
  /v1/hr/payroll-runs/{id}/approve:
    post:
      summary: Finance Officer approves payroll run
      security: [{FinanceOfficerJWT: []}]
  /v1/hr/payroll-runs/{id}/submit:
    post:
      summary: Submit to M-Pesa Bulk API
      security: [{FinanceOfficerJWT: []}]
  /v1/hr/payroll-runs/{id}/lines:
    get:
      summary: Per-staff payroll detail
      security: [{HROfficerJWT: []}]
  /v1/hr/payslip/{line_id}:
    get:
      summary: Staff member payslip (OTP-gated)
      security: [{OTPAuth: []}]
  /v1/hr/kra-return/{run_id}:
    post:
      summary: Submit PAYE P9 return to KRA iTax
      security: [{HROfficerJWT: []}]
```

## 8. Integrations

- **M38 LMS** — certification status gate for contract eligibility
- **M33 Personnel Dispatch** — receives confirmed contracted staff for deployment; M33 reads active contracts from M55 roster
- **M03 ABIS** — biometric conflict check for applicants
- **M12 CRMS** — candidate registry conflict check (Elections Act §27)
- **M49 Referendum Mechanics** — subsidiary staffing campaign for referendum
- **M50 By-Election Operations** — subsidiary staffing campaign per by-election
- **M54 Election Budget** — actual payroll cost feeds into personnel budget line
- **M-Pesa Bulk API** (Safaricom) — bulk disbursement of net pay
- **KRA iTax API** (external) — PAYE return and payment
- **NSSF Portal API** (external) — contribution returns
- **NHIF/SHIF Portal API** (external) — contribution returns
- **M52 Election Calendar + Gazette** — subscriber: receives staff activation event from gazette notice

## 9. UI/UX Key Screens

1. **Recruitment Portal (public)** — bilingual vacancy listing; multi-step application form; phone-number OTP verification; application reference number issued.
2. **HR Dashboard** — applicant funnel by stage; LMS certification heat map by county and role; conflict-flag queue.
3. **Contract Generation Console** — bulk contract generation per role-county batch; status tracker; resend button for bounced emails.
4. **Payroll Run Workspace** — phase selector; staff count and cost preview; deduction summary; approval button; M-Pesa submission status; failed-payment list with re-run button.
5. **Payslip Portal (staff)** — phone-number + OTP access; PDF download; contact HR link for queries.
6. **Statutory Returns Dashboard** — KRA, NSSF, SHIF submission status per payroll run; due dates; confirmation references.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Temporary employee conflict prohibition | Elections Act 2011 §27 |
| Employment contract requirements | Employment Act 2007 §9, §35 |
| PAYE withholding and return | Income Tax Act Cap 470; KRA Regulations |
| NSSF contributions | NSSF Act 2013 (amended 2023) |
| NHIF/SHIF contributions | NHIF Act 1998; SHIF Act 2023 |
| Staff personal data processing | DPA 2019 s.30(1)(b) |
| DPIA for bulk staff data | DPA 2019 s.31; ODPC DPIA Guidelines |
| Payroll record retention | PFM Act 2012; KRA — 7 years |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Payroll computation time (250K staff) | <= 4 h | Payroll run log |
| M-Pesa disbursement completion | <= 6 h from approval | M-Pesa batch log |
| Failed payment rate | < 2% | M-Pesa batch report |
| KRA PAYE return submitted on time | 100% within 5 days of run | iTax submission log |
| LMS gate block rate (uncertified applicants) | Measured and reported | HR dashboard |
| Conflict-flag detection accuracy | 100% (zero false negatives) | ABIS + CRMS cross-check log |

## 12. Test Strategy

- **Unit:** PAYE tax band computation; NSSF/SHIF deduction calculation; per-diem formula; M-Pesa payment file generation format.
- **Integration:** LMS certification check; M12 CRMS conflict check; M-Pesa Bulk API mock; KRA iTax mock submission.
- **E2E:** Full application-to-payslip cycle for a cohort of 1,000 simulated staff.
- **Performance:** Payroll computation for 250,000 staff in <= 4 h on reference hardware.
- **Security:** Confirm staff member can only access their own payslip via OTP; no cross-applicant data leakage.
- **Accessibility:** WCAG 2.2 AA on public recruitment portal and payslip page.
- **Chaos:** M-Pesa API timeout — verify failed-payment list correctly populated and re-run workflow functions.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| M-Pesa Bulk API integration and MoU | 2026-11-01 | Safaricom commercial agreement |
| KRA iTax, NSSF, SHIF API integrations | 2027-01-01 | Statutory portal connections |
| Recruitment portal live | 2027-03-01 | Vacancy notices published for Polling Clerks |
| LMS gate integration tested | 2027-03-15 | M38 certification sync |
| First payroll dry-run (1,000 staff) | 2027-05-01 | Pilot county |
| Full 250K payroll run capacity test | T-60 (2027-06-11) | Load test |
| Polling Day payroll run | T+2 (2027-08-12) | First live disbursement |

## 14. Dependencies

- M38 LMS (certification gate)
- M33 Personnel Dispatch (deployment roster consumer)
- M03 ABIS (biometric conflict check)
- M12 CRMS (candidate conflict check)
- M49 Referendum Mechanics (referendum staffing)
- M50 By-Election Operations (by-election staffing)
- M54 Election Budget (cost feed)
- M-Pesa Bulk API (Safaricom)
- KRA iTax API
- NSSF Portal API
- NHIF/SHIF Portal API

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| M-Pesa Bulk API failure on payday | Low | Critical | Fallback: bank transfer for failed staff via Commission Finance; manual RTGS as last resort; Safaricom SLA in contract |
| KRA tax band change mid-election cycle | Medium | Medium | Tax engine reads rates from configurable table; update table without code deploy; legal team monitors Finance Act |
| NSSF Act 2023 contribution calculation dispute | Medium | Medium | Statutory calculation logic reviewed by certified auditor before go-live; built-in rate versioning |
| Applicant national ID fraud (fake ID) | Medium | High | NRB real-time ID verification at application; biometric at contract collection point for POs and above |
| Staff personal data breach (payroll file) | Low | Critical | Payroll files AES-256 at rest; mTLS to M-Pesa; no plaintext PII in logs; DPA 2019 breach response plan in DPIA |

## 16. Cross-References

- `M33-mobile-personnel-dispatch.md` — deployment logistics consumer of M55 roster
- `M38-lms.md` — certification gate (assumed M38 per LMS reference)
- `M49-referendum-mechanics.md` — referendum staffing campaign
- `M50-by-election-operations.md` — by-election staffing
- `M54-election-budget-expenditure.md` — personnel cost feed
- `M52-election-calendar-gazette.md` — staff activation event subscriber

## 17. Strategy Traceability

- Elections Act 2011 §27 — temporary staff conflict of interest prohibition
- Employment Act 2007 §9, §35 — written contract and particulars of employment
- NSSF Act 2013 (amended 2023) — statutory pension contributions
- SHIF Act 2023 — successor to NHIF; health contribution
- Income Tax Act Cap 470 — PAYE withholding
- DPA 2019 s.30(1)(b), s.31 — lawful basis and DPIA

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
