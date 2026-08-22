# Nomination fee payment (M47)

Module M47 (`nomination-fee-payment`) owned by `crms-svc`. Manages fee schedules per contest tier, M-Pesa Daraja C2B intake with per-TransID idempotent deduplication, B2C refunds, waivers on gender/disability/youth grounds, and end-of-window reconciliation. Hard dependency on M12 (candidate record must exist before fee is accepted).

## Contents

1. What M47 does (vs M12 candidate management)
2. Data plane (schema)
3. Fee schedule and payment modes
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M47 does (vs M12 candidate management)

- **M12** = candidate registration and nomination record. A candidate must exist in M12 before M47 will accept a payment.
- **M47** = the financial layer: fee schedule, M-Pesa intake, waiver adjudication, B2C refund, and end-of-window reconciliation report. It does not create candidates; it confirms that the financial prerequisite for nomination is met and emits `eel.crms.nomination_fee_paid.v1` which M12 listens to in order to unlock the `PAYMENT_CONFIRMED` status on the candidate record.

## 2. Data plane

Five aggregates (per `services/crms-svc/crates/crms-domain/src/models/nomination_fee.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **FeeSchedule** | Configurable fee table: `schedule_id` (UUID7), `election_cycle`, `contest_type` (`PRESIDENT / SENATOR / GOVERNOR / MP / WOMEN_REP / MCA`), `amount_kes`, `currency` (`KES`), `effective_from`, `effective_until`, `gazette_notice_ref`. | B |
| **Payment** | Incoming C2B payment: `payment_id` (UUID7), `candidate_id` (FK to M12), `daraja_trans_id` (unique index; dedup key), `amount_kes`, `phone_number_hash`, `paybill_account_ref`, `received_at`, `status` (`PENDING / CONFIRMED / REFUNDED / WAIVED`), `fee_schedule_id`. | A |
| **Waiver** | Approved fee waiver: `waiver_id` (UUID7), `candidate_id`, `grounds` (`GENDER / DISABILITY / YOUTH`), `supporting_doc_s3_key`, `applied_at`, `officer_decision` (`APPROVED / REJECTED`), `decided_at`, `decided_by`. | B |
| **Refund** | B2C refund record: `refund_id` (UUID7), `payment_id`, `daraja_b2c_trans_id`, `amount_kes`, `initiated_at`, `daraja_result_code`, `completed_at`, `status` (`INITIATED / COMPLETED / FAILED`). | A |
| **ReconciliationReport** | End-of-window summary: `report_id` (UUID7), `election_cycle`, `total_payments`, `total_amount_kes`, `total_waivers`, `total_refunds`, `outstanding_count`, `generated_at`, `signed_by` (Treasury officer). | B |

## 3. Fee schedule and payment modes

| Payment channel | Description | Dedup key |
|---|---|---|
| M-Pesa Daraja C2B (Paybill) | Candidate pays to IEBC paybill; webhook received by Daraja adapter (`services/crms-svc/py/daraja/`) | `daraja_trans_id` (globally unique per Safaricom) |
| Officer manual entry | Officer records offline payment (bank slip) via console | Manual; officer generates internal `payment_id`; no Daraja trans_id |

Daraja webhook handling is idempotent: the `daraja_trans_id` is stored in a unique index. A duplicate webhook is detected and returns `200 OK` with the existing `payment_id` without creating a new `Payment` row. This is the per-TransID dedup prescribed in the Safaricom Daraja API documentation (v3.0).

M-Pesa B2C refunds are initiated via the Daraja B2C API. Each refund attempt stores the `daraja_b2c_trans_id`; a failed B2C result triggers a retry via the Daraja result callback (with exponential backoff, max 3 attempts).

## 4. Pipeline

```
candidate exists in M12 (hard dependency)
      -> Daraja C2B webhook received at /v1/fees/daraja/callback
            -> HMAC-SHA256 webhook signature verified (Daraja secret)
            -> daraja_trans_id checked against unique index (dedup)
            -> if new: Payment created (status = PENDING)
            -> fee_schedule_id resolved from contest_type + election_cycle
            -> amount_kes validated against schedule
            -> if amount matches: status -> CONFIRMED
            -> if amount < schedule: OvershorPayment alert; status stays PENDING
            -> emit eel.crms.nomination_fee_paid.v1 (on CONFIRMED)
            -> M12 sets candidate PAYMENT_CONFIRMED
      -> waiver workflow (parallel path):
            -> candidate applies via portal; Waiver created
            -> officer reviews supporting document
            -> if APPROVED: Payment created with status = WAIVED (amount = 0)
            -> emit eel.crms.nomination_fee_paid.v1 (waiver path)
      -> refund (candidate withdraws or is disqualified):
            -> Refund created; Daraja B2C API called
            -> on daraja result callback: status -> COMPLETED or FAILED
            -> if FAILED: retry with exponential backoff (1 min, 2 min, 4 min)
            -> Payment status -> REFUNDED on Refund COMPLETED
      -> end-of-window: ReconciliationReport generated
            -> total_payments / waivers / refunds / outstanding reconciled
            -> Treasury officer signs; stored in S3 WORM
```

## 5. API surface

Contract: `packages/openapi-contracts/crms/nomination.yaml` (under `nomination` path in contracts listing). All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/fees/daraja/callback` | Daraja system (Safaricom IP allowlist) | C2B webhook intake |
| `POST /v1/fees/daraja/b2c/result` | Daraja system (Safaricom IP allowlist) | B2C result callback |
| `GET /v1/fees/payments/{candidate_id}` | Candidate (OIDC) or officer | Payment status for candidate |
| `POST /v1/fees/payments` | Officer | Manual payment entry (bank slip) |
| `POST /v1/fees/waivers` | Candidate (OIDC) | Apply for waiver + document upload |
| `PATCH /v1/fees/waivers/{waiver_id}` | RPP officer | Approve or reject waiver |
| `POST /v1/fees/refunds` | Officer | Initiate B2C refund |
| `GET /v1/fees/schedule` | Public (60/hr) | Current fee schedule by contest type |
| `GET /v1/fees/reconciliation` | Treasury officer | Latest ReconciliationReport |
| `POST /v1/fees/reconciliation` | System (end-of-window cron) | Generate ReconciliationReport |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Candidate payment portal (Astro) | `services/crms-svc/astro/party-registration` (shared) | Candidate views payment status; applies for waiver |
| RPP officer console (LiveView) | `services/crms-svc/ex/rpp_web` | Officer reviews waivers; records manual payments; initiates refunds |
| Daraja adapter (FastAPI) | `services/crms-svc/py/daraja/` | Webhook receiver; not a UI but a machine endpoint |
| Treasury reconciliation panel (Django) | `services/crms-svc/django/governance-admin` | Treasury officer reviews + signs ReconciliationReport |

Languages: EN + SW on candidate portal. Officer surfaces: EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Per-TransID dedup** - `daraja_trans_id` unique index prevents double-crediting from Daraja webhook retries. The Daraja adapter returns `200 OK` on duplicates (as required by Safaricom's webhook retry contract) but does not create a new `Payment` row.
- **Webhook HMAC verification** - every inbound Daraja callback is verified against the HMAC-SHA256 signature using the Safaricom API secret stored in Vault (OpenBao on 62.169.25.77). An invalid signature returns `401` and is logged in `eel_crms_audit_shadow`.
- **Fee schedule gazette linkage** - each `FeeSchedule` row carries a `gazette_notice_ref`; the API rejects a fee schedule with no gazette reference. This ensures fee changes are publicly notified before taking effect.
- **Waiver document WORM** - supporting documents for waivers are stored in S3 with object-lock. Officer rejection can be appealed; the original document is preserved.
- **Reconciliation immutability** - `ReconciliationReport` rows are insert-only; no UPDATE. A corrected reconciliation creates a new report with a `supersedes_report_id` reference.
- **B2C retry cap** - the Daraja B2C adapter retries a failed refund at most 3 times. If all 3 fail, a `Refund.status = FAILED` is set and an alert is sent to Treasury; manual resolution is required (documented in runbook).

## 8. Kenya statute alignment

- **Elections Act §24** - nomination fees prescribed by regulation; M47's `FeeSchedule` with gazette reference operationalises the statutory source.
- **Elections (General) Regulations 2012, Reg 47** - fee amounts per contest tier; M47's schedule is configurable to match the gazette-notified amounts.
- **Gender Act 2006 / Constitution Art 27** - gender-based waiver aligns with the affirmative action mandate.
- **Persons with Disabilities Act 2003** - disability waiver aligns with the statutory protection.
- **Kenya Revenue Authority Act** - all fees are recorded in the ReconciliationReport for Treasury reporting obligations.
- **National Payment System Act 2011** - M-Pesa Daraja operates under Central Bank of Kenya payment system regulation; M47 uses the Daraja API in accordance with Safaricom's licensed operation.

## 9. Honest caveats + external prerequisites

- Daraja C2B paybill number and API credentials must be provisioned by IEBC via Safaricom's business API onboarding. The adapter at `services/crms-svc/py/daraja/` is ready; the live credentials are an external dependency stored in Vault.
- Daraja B2C refund requires IEBC to maintain a Safaricom B2C operator account with sufficient float. The reconciliation report tracks refund liability; Treasury must ensure float is maintained.
- Bank-slip manual payments are not validated against any bank API. Officer entry is the only control. A future integration with IEBC's banker for real-time confirmation is noted as a gap.
- The waiver supporting document review relies on officer judgment for disability and gender grounds. There is no integration with NCPWD (National Council for Persons with Disabilities) or KDPA for automated verification.

## 10. Test coverage + operational runbooks

Tests under `services/crms-svc/tests/`: Daraja webhook idempotency via `pytest-httpserver`, B2C retry arithmetic via `proptest`, waiver lifecycle integration tests, reconciliation arithmetic.

Runbooks under `docs/runbooks/`:

- `crms-svc.md` - Daraja adapter deploy + rollback; webhook secret rotation.
- `b2c-refund-failure.md` - manual refund resolution when all 3 Daraja retries fail.
- `daraja-down.md` - payment intake degraded mode (manual entry fallback).

Related documents: spec at `docs/plans/modules/nomination-fee-payment.md`; M12 at `docs/features/`; capability catalogue at `services/crms-svc/docs/CAPABILITIES.md`.
