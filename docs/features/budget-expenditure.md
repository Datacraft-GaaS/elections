# Budget and expenditure (M54)

Module M54 (`budget-expenditure`) owned by `support-admin-svc`. Manages the Commission election budgets, procurement records, payment vouchers, and Controller of Budget (CoB) / OAG reports; integrates with IFMIS and the PPRA e-procurement portal; enforces separation-of-duties (SoD) on every approval step.

## Contents

1. What Budget and Expenditure does (vs external IFMIS)
2. Data plane (schema)
3. External integrations
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Budget and Expenditure does (vs external IFMIS)

- **IFMIS** (Government Integrated Financial Management Information System) = the National Treasury's authoritative commitment and payment ledger; M54 cannot replace it.
- **M54** = the Commission's internal budget planning, line-item tracking, procurement workflow, and statutory reporting layer that feeds into IFMIS and generates CoB quarterly reports.

M54 is the pre-commitment control plane: it enforces SoD before any payment reaches IFMIS and produces the audit trail that the OAG and CoB require.

## 2. Data plane

Five aggregates (per `services/support-admin-svc/crates/budget/src/models.rs` and the M54 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **Budget** | Approved annual or supplementary budget: `budget_id`, `financial_year`, `programme_code`, `line_items` (JSONB array with `item_code`, `description`, `approved_amount`, `currency`), `status` (DRAFT / SUBMITTED / APPROVED / SUPPLEMENTARY), commissioner + treasury HSM countersignature. | A |
| **Procurement** | Procurement record per PPRA procedure: `proc_id`, `budget_line_item_id`, `method` (OPEN_TENDER / RESTRICTED / DIRECT / QUOTATION), `tender_ref`, `awarded_to`, `awarded_amount`, `ppra_confirmation_number`. | A |
| **PaymentVoucher** | Individual payment instruction: `voucher_id`, `procurement_id`, `payee_name`, `amount`, `ifmis_transaction_id`, `recommended_by`, `approved_by`, `authorized_by` (three distinct officers, SoD enforced). | A |
| **CoBReport** | Quarterly CoB expenditure report: `report_id`, `quarter`, `total_expenditure`, `variance_to_budget`, `narrative_md`, `submitted_at`, `cob_reference_number`. | B |
| **SodViolationLog** | Immutable log of SoD attempts: `log_id`, `voucher_id`, `attempted_by`, `violation_type`, `detected_at`. | A |

## 3. External integrations

| Integration | Direction | Auth | SLA | Failure handling |
|---|---|---|---|---|
| **IFMIS** | Outbound (payment instructions) | OAuth2 client credentials (National Treasury IdP) | 2 h acknowledgement | Retry 5x; manual escalation if unacknowledged after 24 h |
| **PPRA e-procurement portal** | Outbound (tender publication) + Inbound (award confirmation) | PPRA API key | Per PPRA SLA | Retry; flag procurement for manual PPRA entry |
| **CoB reporting portal** | Outbound (quarterly reports) | CoB client certificate | Quarterly deadline | Retry; alert commissioner office 72 h before deadline |
| **OAG audit interface** | Inbound read-only scoped access (see M59) | SVID mTLS | On-demand | See M59 scope guards |

## 4. Pipeline

Files on disk under `services/support-admin-svc/crates/budget/src`: `lib.rs`, `models.rs`, `procurement.rs`, `voucher.rs`, `cob_report.rs`, `routes/`.

```
budget officer drafts Budget (line items + amounts)
      -> CFO recommends; commissioner approves + HSM signature
      -> emit eel.admin.budget_approved.v1
      -> procurement officer creates Procurement record
            -> PPRA method selected per threshold rules
            -> tender published to PPRA portal
            -> award received: PPRA confirmation number stored
      -> accounts officer drafts PaymentVoucher
            -> SoD check: recommended_by != approved_by != authorized_by
            -> all three must hold distinct SpiceDB roles:
               ACCOUNTS_OFFICER / BUDGET_APPROVER / CFO
            -> SodViolationLog written if any check fails
      -> authorized PaymentVoucher synced to IFMIS
            -> emit eel.admin.payment_sent_to_ifmis.v1
            -> IFMIS returns transaction ID; stored in voucher
      -> quarterly: CoBReport assembled from vouchers + variance
            -> submitted to CoB portal; reference number stored
            -> emit eel.admin.cob_report_submitted.v1
```

Every step uses the transactional outbox (`packages/eel-outbox-rs`).

## 5. API surface

Contract: `packages/openapi-contracts/support-admin/budget.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/budgets` | Budget officer (JWT + SpiceDB `BUDGET_OFFICER`) | Create draft budget |
| `PATCH /v1/budgets/{id}/approve` | Commissioner + HSM | Approve budget |
| `POST /v1/budgets/{id}/procurements` | Procurement officer | Add procurement record |
| `POST /v1/budgets/{id}/vouchers` | Accounts officer | Draft payment voucher |
| `PATCH /v1/budgets/{id}/vouchers/{vid}/recommend` | Budget approver | Recommend voucher |
| `PATCH /v1/budgets/{id}/vouchers/{vid}/approve` | CFO | Approve voucher; triggers IFMIS sync |
| `GET /v1/budgets/{id}/vouchers` | OAG / Commission admin | List vouchers + IFMIS status |
| `POST /v1/cob-reports` | Accounts officer | Draft quarterly CoB report |
| `PATCH /v1/cob-reports/{id}/submit` | CFO + HSM | Submit to CoB portal |
| `GET /v1/sod-violations` | Internal audit (JWT + SpiceDB `INTERNAL_AUDIT`) | SoD violation log |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Budget planning console (Django) | `services/support-admin-svc/django/budget_admin` | Draft, review, approve budgets |
| Procurement workflow (Django) | `services/support-admin-svc/django/budget_admin` | Manage tenders, record awards |
| Voucher approval workspace (LiveView) | `services/support-admin-svc/ex/voucher_web` | Three-officer SoD approval flow |
| CoB report builder (Django) | `services/support-admin-svc/django/budget_admin` | Assemble and submit quarterly reports |

Languages: EN only (internal staff). WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **SoD enforcement at the DB layer**: a Postgres check constraint on `payment_vouchers` ensures `recommended_by`, `approved_by`, and `authorized_by` are three distinct user UUIDs; the application layer adds role checks via SpiceDB.
- **Immutable SodViolationLog**: rows are insert-only; no update or delete permission is granted to any application role.
- **IFMIS transaction ID linkage**: a voucher with `status = SENT_TO_IFMIS` must have a non-null `ifmis_transaction_id`; a nightly integrity job fails CI if any such rows are missing the ID.
- **HSM-signed budget approval**: the approved `Budget` carries a commissioner HSM signature over the SHA-256 of the line-item JSONB; any post-approval tampering is detectable.
- **Audit shadow**: every state transition of every aggregate is logged to `eel_admin_audit_shadow` with officer identity, IP, and timestamp.
- **OAG scope guards** (see M59): the OAG read-only view is time-boxed and scoped to the agreed engagement period; no voucher content is exposed outside those bounds.

## 8. Kenya statute alignment

- **Public Finance Management Act 2012 §68 (expenditure controls)**: the three-officer voucher approval map directly to the PFM Act's commitment, verification, and authorisation steps.
- **Public Finance Management Act 2012 §71 (budget management)**: budget line items are structured to the Government of Kenya programme-based budget codes.
- **PPRA Act 2015 (procurement thresholds)**: procurement method selection is rule-driven against statutory KES thresholds; threshold table is versioned in `services/support-admin-svc/config/ppra_thresholds.toml`.
- **Controller of Budget Act 2016 §33 (quarterly reports)**: `CoBReport` maps to the statutory CoB expenditure return format.
- **Kenya Data Protection Act §26 (minimisation)**: payee banking details are stored encrypted at rest (AES-256-GCM); only the CFO role can decrypt for IFMIS submission.

## 9. Honest caveats + external prerequisites

- IFMIS integration depends on National Treasury provisioning an OAuth2 client for the Commission. The adapter is written to the IFMIS REST API documented version; breaking changes in Treasury's API require adapter updates.
- PPRA portal integration uses the PPRA Open Contracting Data Standard (OCDS) adapter; if PPRA changes their schema the adapter must be updated before procurement records can be submitted.
- CoB reporting format is modelled on the 2022 CoB circular; if CoB revises the format, the report builder must be updated before the next filing deadline.
- The three-officer SoD rule is configurable (threshold for which voucher amounts require all three vs. two) but the Commission CFO must formally set these thresholds in `config/sod_thresholds.toml` before go-live.
- No integration with the National Treasury's Single Treasury Account (STA) for cash management; M54 tracks commitments and payments but does not manage the bank account.

## 10. Test coverage + operational runbooks

Coverage under `services/support-admin-svc/crates/budget/tests/`: SoD enforcement (property-based, all permutations of officer role assignments), IFMIS sync round-trip (pytest-httpserver mock), CoB report generation (golden-file tests against 2022 circular format), PPRA threshold selection (table-driven).

Runbooks under `docs/runbooks/`:

- `support-admin-svc.md` (section: budget-expenditure) -- deploy, rollback, IFMIS sync drain.
- `cob-deadline-escalation.md` -- steps when CoB portal is unreachable within 72 h of deadline.
- `ifmis-sync-failure.md` -- manual IFMIS reconciliation when automated sync fails.

Related documents: full spec at `docs/plans/modules/budget-expenditure.md`; service capability catalogue at `services/support-admin-svc/docs/CAPABILITIES.md`.

### Key metrics and SLOs

| Metric | Target | Alert threshold |
|---|---|---|
| IFMIS sync success rate | >= 99% | < 99%: alert to payable officer |
| CoB report submission lead time | >= 5 days before deadline | < 3 days: warning |
| SoD violation attempt rate | 0 per quarter | Any attempt: immediate audit alert |
| Payment voucher end-to-end approval time | <= 5 business days | > 7 days: escalate to CFO |

### SoD role matrix

The three-officer approval chain maps to distinct Postgres-level roles and SpiceDB permissions:

| Step | SpiceDB role | Postgres role | Cannot overlap with |
|---|---|---|---|
| Recommend | `ACCOUNTS_OFFICER` | `accounts_writer` | approved_by, authorized_by |
| Approve | `BUDGET_APPROVER` | `budget_approver` | recommended_by, authorized_by |
| Authorize | `CFO` | `cfo_authorizer` | recommended_by, approved_by |

The constraint is enforced both at the application layer (SpiceDB) and at the database layer (Postgres `CHECK` constraint). Breaking one layer still fails at the other.
