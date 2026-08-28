# M47 — Nomination Fee Payment

**Module ID:** M47
**Name:** Nomination Fee Payment
**Runs on:** Central servers (Kenyan cloud) + candidate-facing portal
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in IEBC per GCC 15.4

---

## 1. Purpose

Manage candidate nomination fee collection, reconciliation, refund, and waiver per Elections Act s.22 and the IEBC Nomination Fee Regulations. Payment channels: M-Pesa Paybill and bank transfer. Waiver process for indigent candidates. Integration with M12 CRMS (nomination clearance gate) and M45 Party Registration (party status gate). All payment events reconciled against IEBC designated account; receipts issued; refund workflow for eliminated candidates.

Strategy reference: §XXI.B (Candidate Nomination). Elections Act s.22; IEBC Nomination Fee Regulations 2022.

---

## 2. User Personas

- **Candidate** — pays nomination fee; requests waiver; downloads receipt; tracks refund
- **Party Official** — pays fees on behalf of multiple candidates from party account
- **IEBC Finance Officer** — reconciles payments against IEBC designated account; approves refunds; reviews waiver requests
- **IEBC Returning Officer** — confirms fee payment as part of M12 nomination clearance
- **IEBC DPO** — oversees DPA 2019 compliance for payment data
- **Auditor** — reviews payment and refund reconciliation reports

---

## 3. Functional Requirements

### Fee Schedule

- **FR-M47-001** Fee schedule configured per contest tier (President, Governor, Senator, MP, CWR, MCA) per IEBC gazette notice; schedule version-controlled; active schedule drives all payment validations.
- **FR-M47-002** Fee schedule published on public portal (no login required) with gazette reference and effective date.

### Payment

- **FR-M47-003** M-Pesa Paybill payment: candidate initiates payment on portal; portal generates a unique payment reference (UUID7-based short code); candidate pays via M-Pesa Paybill using reference; Daraja API callback confirms payment; reference matched to candidate application in M12.
- **FR-M47-004** Bank transfer payment: portal generates a unique pay-in reference; candidate presents to bank; IEBC Finance Officer manually confirms receipt in back-office within 2 working days; candidate notified.
- **FR-M47-005** Payment status: PENDING → CONFIRMED / FAILED / EXPIRED (reference expires 24 hours after generation for M-Pesa; 5 working days for bank transfer).
- **FR-M47-006** Receipt issuance: on CONFIRMED, system generates a digitally signed PDF receipt (fee amount, contest, candidate name, NIN, date, IEBC reference, HSM signature); receipt emailed and available for download.
- **FR-M47-007** Party bulk payment: party official generates a batch of payment references for multiple candidates; pays total via single bank transfer; IEBC Finance Officer splits and allocates per candidate on confirmation.
- **FR-M47-008** M12 clearance gate: M12 nomination workflow queries M47 API for CONFIRMED status before advancing candidate to DOCS_UPLOADED step; unpaid candidates blocked.
- **FR-M47-009** Payment window enforcement: payment references can only be generated within the gazetted nomination period; late-window requests rejected with statutory reference.

### Waiver

- **FR-M47-010** Waiver application: candidate submits waiver request with grounds (financial hardship, disability, youth category per IEBC Regulations) and supporting documents (means assessment, disability certificate).
- **FR-M47-011** Waiver review: IEBC Finance Officer reviews within 5 working days; approves, partially approves (fee reduction), or rejects with written reason.
- **FR-M47-012** Waiver decision notification: candidate notified with decision letter (signed PDF); if approved, M47 marks payment as WAIVED; M12 gate accepts WAIVED status.
- **FR-M47-013** Waiver register: IEBC publishes anonymised waiver grant statistics (count by contest tier, county) post-nomination window; no individual names published.

### Refund

- **FR-M47-014** Refund eligibility: candidate whose nomination is rejected by RO (M12 clearance denied) is eligible for a full fee refund per Elections Act s.22(4); candidate who withdraws before gazette is eligible for 50% refund per IEBC Regulations.
- **FR-M47-015** Refund workflow: RO triggers refund eligibility event in M12; M47 creates refund record; IEBC Finance Officer reviews and approves; payment returned via M-Pesa reverse or bank transfer within 10 working days.
- **FR-M47-016** Refund receipt: digitally signed PDF refund confirmation issued to candidate.
- **FR-M47-017** Refund reconciliation: Finance Officer reconciles all refunds against IEBC designated account within T+30 (30 days post-election); reconciliation report archived.

---

## 4. Non-Functional Requirements

- **NFR-M47-001** M-Pesa Daraja callback processing: <= 5 seconds end-to-end (payment initiation to CONFIRMED status).
- **NFR-M47-002** Portal p99 <= 1 s under 2,000 concurrent users during nomination window.
- **NFR-M47-003** Receipt generation: <= 30 seconds of CONFIRMED event.
- **NFR-M47-004** WCAG 2.2 AA for candidate-facing portal; EN + SW.
- **NFR-M47-005** DPA 2019 s.30(1)(b) lawful basis (legal obligation); payment data encrypted at rest (AES-256); M-Pesa phone number stored only for refund routing; retained 7 years post-election; DPIA registered.
- **NFR-M47-006** Availability >= 99.9% during nomination window (typically 2 weeks, T-60 to T-46).
- **NFR-M47-007** All payment events audit-logged; appended to P05 Merkle chain.
- **NFR-M47-008** Daraja API idempotency: duplicate Daraja callbacks handled without double-crediting.

---

- **NFR-47-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-47` (replace 47 with the module number); alert ruleset ID `iems-47-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a candidate, I generate an M-Pesa payment reference on the portal, pay via my phone, and receive a digitally signed PDF receipt in my email within 2 minutes.
2. As a candidate in financial hardship, I submit a waiver application with a means-assessment document and receive an IEBC decision within 5 working days.
3. As a party official, I generate payment references for all 15 of our county candidates in a batch and pay a single bank transfer; each candidate's receipt is generated when the Finance Officer allocates the payment.
4. As an IEBC Finance Officer, I see a reconciliation dashboard comparing IEBC designated account deposits against confirmed portal payments, with any unmatched items flagged.
5. As a candidate whose nomination was rejected by the RO, I receive a refund initiation notice within 24 hours and the money within 10 working days.
6. As the M12 CRMS workflow, I query the M47 API and confirm payment status is CONFIRMED or WAIVED before advancing a candidate's nomination to the document review step.
7. As an IEBC Returning Officer, I see payment status on the M12 nomination card without leaving the clearance workflow.
8. As an auditor, I download the end-of-cycle reconciliation report showing every payment, waiver, and refund with P05 Merkle path for integrity verification.
9. As a candidate who withdrew before the gazette, I apply for the 50% partial refund online and track its status to completion.
10. As a member of the public, I view the fee schedule on the IEBC portal with the gazette reference and effective date, so I know exactly what a candidate must pay for each position.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import date, datetime
from enum import Enum
from decimal import Decimal

class PaymentStatus(str, Enum):
    PENDING = "PENDING"
    CONFIRMED = "CONFIRMED"
    FAILED = "FAILED"
    EXPIRED = "EXPIRED"
    WAIVED = "WAIVED"
    REFUNDED = "REFUNDED"

class FeeSchedule(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    gazette_reference: str
    effective_date: date
    contest_tiers: list[dict]          # [{tier, fee_kes}]
    active: bool

class NominationFeePayment(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    candidate_id: str                  # FK to M12 CRMS
    party_id: str | None               # FK to M45, if party-paid
    contest_tier: str
    fee_amount_kes: Decimal
    payment_reference: str             # UUID7-based unique code
    payment_channel: str               # MPESA / BANK_TRANSFER
    mpesa_transaction_id: str | None
    bank_reference: str | None
    status: PaymentStatus
    confirmed_at: datetime | None
    receipt_url: str | None
    waiver_id: str | None
    created_at: datetime
    updated_at: datetime

class WaiverApplication(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    candidate_id: str
    grounds: str
    supporting_docs: list[str]
    status: str                        # PENDING / APPROVED / PARTIAL / REJECTED
    approved_amount_kes: Decimal | None
    decision_reason: str | None
    decided_by: str | None
    decided_at: datetime | None
    created_at: datetime

class RefundRecord(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    payment_id: str
    refund_amount_kes: Decimal
    refund_basis: str                  # NOMINATION_REJECTED / WITHDRAWAL_50PCT
    status: str                        # PENDING / APPROVED / PAID
    paid_at: datetime | None
    receipt_url: str | None
    created_at: datetime
```

---

## 7. API (OpenAPI 3.1 excerpt)

```yaml
openapi: "3.1.0"
info:
  title: M47 Nomination Fee Payment API
  version: "1.0.0"
paths:
  /api/v1/fee-schedule:
    get:
      summary: Get active fee schedule (public)
      responses:
        "200": { description: "FeeSchedule with contest tier breakdown" }
  /api/v1/payments:
    post:
      summary: Initiate nomination fee payment
      security: [{ BearerAuth: [candidate, party_official] }]
      responses:
        "201": { description: "Payment reference generated" }
  /api/v1/payments/{id}/status:
    get:
      summary: Check payment status (M12 integration endpoint)
      security: [{ BearerAuth: [candidate, crms_service] }]
      responses:
        "200": { description: "PaymentStatus with confirmed_at" }
  /api/v1/callbacks/mpesa:
    post:
      summary: M-Pesa Daraja STK callback (internal; IP-allowlisted)
      responses:
        "200": { description: "Callback acknowledged" }
  /api/v1/waivers:
    post:
      summary: Submit waiver application
      security: [{ BearerAuth: [candidate] }]
      responses:
        "202": { description: "Waiver application received" }
  /api/v1/refunds:
    post:
      summary: IEBC triggers refund eligibility
      security: [{ BearerAuth: [finance_officer, returning_officer] }]
      responses:
        "201": { description: "Refund record created" }
  /api/v1/reconciliation:
    get:
      summary: Finance Officer reconciliation report
      security: [{ BearerAuth: [finance_officer, auditor] }]
      responses:
        "200": { description: "Reconciliation CSV/JSON" }
```

---

## 8. Integrations

| System | Direction | Purpose |
|---|---|---|
| M-Pesa Daraja API (Safaricom) | Inbound/Outbound | STK push initiation; payment callback; refund reverse |
| M12 CRMS | Outbound | Payment status gate; refund eligibility trigger from RO |
| M45 Party Registration | Inbound | Party ACTIVE status required for party bulk payment |
| P05 Merkle Log | Outbound | Payment event immutable audit |
| P06 HSM | Outbound | Receipt and refund-letter signing |
| IEBC Designated Bank (CBK MoU) | Outbound | Bank transfer reconciliation feed |

---

## 9. UI/UX

- Candidate portal: fee-schedule view; payment-channel selector; M-Pesa STK push initiation with countdown timer; status polling widget; receipt download.
- Waiver form: grounds dropdown (Financial Hardship / Disability / Youth Category); document upload; status tracker.
- Party bulk-payment portal: candidate roster table; batch reference generation; Finance Officer allocation interface.
- IEBC Finance Officer back-office: payment queue with unmatched-item flags; reconciliation export; waiver review panel; refund approval modal.
- All candidate-facing screens WCAG 2.2 AA; EN + SW.

---

## 10. Compliance Mapping

| Requirement | Provision | Implementation |
|---|---|---|
| Fee schedule by gazette | Elections Act s.22(1) | Fee schedule version-controlled with gazette reference |
| Refund on rejected nomination | Elections Act s.22(4) | Automated refund trigger from M12 RO decision |
| Waiver provision | IEBC Nomination Fee Regulations 2022 | Waiver workflow FR-M47-010 to FR-M47-013 |
| DPA 2019 payment data | s.30(1)(b) legal obligation | Encryption, minimal retention, DPIA filed |
| Audit of IEBC funds | PFMA 2012 | Reconciliation report FR-M47-017; Merkle-anchored events |

---

## 11. Success Metrics + SLA

| Metric | Target |
|---|---|
| M-Pesa confirmation time | <= 5 seconds |
| Receipt generation | <= 30 seconds of CONFIRMED |
| Bank transfer confirmation | <= 2 working days |
| Waiver decision turnaround | <= 5 working days |
| Refund processing | <= 10 working days |
| System availability during nomination window | >= 99.9% |
| Unmatched reconciliation items at T+30 | Zero |

---

## 12. Test Strategy

- Unit: Daraja callback idempotency; payment-reference expiry; refund-amount computation.
- Integration: Daraja sandbox; M12 payment-gate query; M45 party status check; P05 Merkle append.
- Load: 2,000 concurrent M-Pesa initiations during nomination-window open.
- Security: reference enumeration; callback spoofing; duplicate callback injection.
- UAT: IEBC Finance Officers; candidate pilot cohort; party officials.

---

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date |
|---|---|
| Daraja API integration agreement with Safaricom | T-270 (2026-11-13) |
| Fee schedule gazette confirmation | T-180 (2027-02-11) |
| Portal UAT with IEBC Finance | T-90 (2027-05-12) |
| Nomination window opens (payment starts) | T-60 (2027-06-11) |
| Nomination window closes | T-46 (2027-06-25) |
| Refund processing complete | T+40 (2027-09-19) |

---

## 14. Dependencies

- Safaricom Daraja API production access + Paybill number assigned to IEBC (T-270).
- M12 CRMS payment-gate API endpoint specified (T-90).
- M45 Party Registration ACTIVE status API available (T-90).
- IEBC designated bank account confirmed for reconciliation feed (T-180).
- Fee schedule gazette notice issued (T-180).

---

## 15. Risks + Mitigations (Top 5)

| Risk | Likelihood | Mitigation |
|---|---|---|
| Daraja API downtime during nomination window | Medium | Bank transfer as fallback; Finance Officer manual confirmation pathway; Daraja SLA MoU |
| Duplicate M-Pesa callbacks causing double-confirmation | Medium | Idempotency key (Daraja transaction ID); duplicate detection before processing |
| Waiver fraud (false hardship claims) | Medium | Supporting document requirement; Finance Officer review; audit trail |
| Nomination window surge causing portal slowdown | High | Auto-scaling; pre-generated reference batch for bulk users; load test at T-14 |
| Bank transfer delay blocking nomination clearance | Medium | M12 allows provisional advance (Finance Officer flag) with 2-day resolution commitment |

---

## 16. Cross-References

- `M12-crms.md` — nomination clearance gate; refund eligibility trigger
- `M45-party-registration-management.md` — party ACTIVE status gate for bulk payment
- `M22-campaign-finance-transparency.md` — nomination fee not counted as campaign spending (confirm with ECFA interpretation)
- `P05-merkle-audit.md` — payment event audit chain
- `P06-hsm.md` — receipt + refund-letter signing

---

## 17. Strategy Traceability

| Strategy Section | Requirement | Module Response |
|---|---|---|
| §XXI.B Candidate Nomination | Fee payment as nomination prerequisite | FR-M47-008 (M12 gate) |
| Elections Act s.22(1) | Fee schedule by gazette | FR-M47-001, FR-M47-002 |
| Elections Act s.22(4) | Refund on rejection | FR-M47-014, FR-M47-015 |
| IEBC Nomination Fee Regulations 2022 | Waiver categories | FR-M47-010 to FR-M47-013 |
| PFMA 2012 | Public funds reconciliation | FR-M47-017 |

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
