# HR and payroll (M55)

Module M55 (`hr-payroll`) owned by `support-admin-svc`. Covers the full lifecycle of the Commission election personnel: job applications, shortlisting, contracts, and payroll runs (250k staff target, completed in 4 hours); M-Pesa B2C batch disbursement; KRA P9/PAYE, NSSF, and NHIF statutory returns; and a training-completion gate that links to M38 (Poll Worker Training LMS).

## Contents

1. What HR and Payroll does (vs M38 Training LMS)
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

## 1. What HR and Payroll does (vs M38 Training LMS)

- **M38** = poll-worker training delivery, assessment, and certification. It owns the training record and emits `eel.training.certification_granted.v1`.
- **M55** = employment lifecycle from application through payroll disbursement. It gates contract issuance on the M38 certification event and gates payroll inclusion on a signed contract.

No staff member is paid without: (a) a passed M38 training assessment and (b) a countersigned `StaffContract`. This gate is enforced in the payroll run pre-flight check.

## 2. Data plane

Five aggregates (per `services/support-admin-svc/crates/hr/src/models.rs` and the M55 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **JobApplication** | Applicant record: `application_id`, `national_id_hash`, `role` (PRESIDING_OFFICER / DEPUTY_PO / POLL_CLERK / etc.), `constituency_code`, `status` (RECEIVED / SHORTLISTED / REJECTED / APPOINTED). | B |
| **ShortlistDecision** | Officer shortlisting decision: `decision_id`, `application_id`, `decided_by`, `basis` (SCORE / QUOTA / APPEAL), `score` (0-100), `decided_at`. | B |
| **StaffContract** | Employment contract: `contract_id`, `application_id`, `role`, `start_date`, `end_date`, `gross_pay_kes`, `m_pesa_number`, `signed_by_applicant_at`, `signed_by_hr_officer_at`, `m38_cert_id` (foreign key to M38 certification). | A |
| **PayrollRun** | Batch payroll computation: `run_id`, `run_date`, `staff_count`, `total_gross_kes`, `total_net_kes`, `status` (DRAFT / APPROVED / DISBURSING / COMPLETE / FAILED), started/completed timestamps, commissioner HSM signature on approval. | A |
| **PayrollLine** | Per-staff pay computation: `line_id`, `run_id`, `contract_id`, `gross_kes`, `paye_kes`, `nssf_kes`, `nhif_kes`, `net_kes`, `mpesa_result_code`, `mpesa_transaction_id`. | A |

## 3. External integrations

| Integration | Direction | Auth | SLA | Failure handling |
|---|---|---|---|---|
| **M38 Training LMS** | Inbound event (`eel.training.certification_granted.v1`) | Kafka SASL/SCRAM | Real-time | Contract creation blocked until cert received; daily reconciliation job flags gaps |
| **M-Pesa B2C API** (Daraja 2.0) | Outbound batch | OAuth2 consumer key + shortcode | Per-call; 250k calls batched over 4 h | Retry on `FAILED` result codes; manual reconciliation for unresolved failures |
| **KRA iTax** | Outbound (P9A forms, PAYE returns) | SVID mTLS (Tier-B integration, WP-4.12) | Monthly / quarterly deadlines | Retry; alert tax officer 72 h before deadline |
| **NSSF portal** | Outbound (contribution schedules) | NSSF API credentials | Monthly | Retry; manual upload fallback |
| **NHIF portal** | Outbound (contribution schedules) | NHIF API credentials | Monthly | Retry; manual upload fallback |

## 4. Pipeline

Files on disk under `services/support-admin-svc/crates/hr/src`: `lib.rs`, `models.rs`, `recruitment.rs`, `payroll.rs`, `statutory_returns.rs`, `routes/`.

```
applicant submits JobApplication via portal (M11) or CSV upload
      -> HR officer shortlists: ShortlistDecision created
      -> M38 certification event received: eel.training.certification_granted.v1
      -> HR officer creates StaffContract (m38_cert_id required)
      -> applicant e-signs contract (OTP via M-Pesa number)
      -> HR officer countersigns
      -> emit eel.hr.contract_signed.v1
      -> payroll officer creates PayrollRun (specifies pay date)
            -> pre-flight: count staff with signed contracts + M38 cert
            -> compute PayrollLine for each: gross - PAYE - NSSF - NHIF = net
               (PAYE: KRA graduated table v2024-07, Waweru et al., KRA 2024)
            -> commissioner approves + HSM signature
            -> emit eel.hr.payroll_approved.v1
            -> M-Pesa B2C batch: 250k lines dispatched in parallel workers
               target: all disbursements initiated within 4 h
            -> poll M-Pesa result callbacks; update PayrollLine.mpesa_result_code
            -> emit eel.hr.payroll_complete.v1 when all lines resolved
      -> monthly: statutory returns submitted to KRA / NSSF / NHIF
```

Payroll computation is CPU-bound; it runs in a dedicated `tokio::task::spawn_blocking` pool to avoid starving the async runtime.

## 5. API surface

Contract: `packages/openapi-contracts/support-admin/hr.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/hr/applications` | Applicant (public JWT) | Submit job application |
| `GET /v1/hr/applications` | HR officer | List applications by role + status |
| `PATCH /v1/hr/applications/{id}/shortlist` | HR officer (SpiceDB `HR_OFFICER`) | Record shortlisting decision |
| `POST /v1/hr/contracts` | HR officer | Create contract (requires M38 cert) |
| `PATCH /v1/hr/contracts/{id}/sign-applicant` | Applicant (OTP-validated) | Applicant e-signature |
| `PATCH /v1/hr/contracts/{id}/sign-hr` | HR officer | HR countersignature |
| `POST /v1/hr/payroll-runs` | Payroll officer | Create payroll run draft |
| `PATCH /v1/hr/payroll-runs/{id}/approve` | Commissioner + HSM | Approve and initiate disbursement |
| `GET /v1/hr/payroll-runs/{id}/lines` | Internal audit (SpiceDB `INTERNAL_AUDIT`) | Per-staff pay lines |
| `GET /v1/hr/payroll-runs/{id}/status` | Payroll officer | Disbursement progress |
| `POST /v1/hr/statutory-returns/{type}` | Tax officer | Trigger KRA/NSSF/NHIF return submission |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Applicant portal (Astro) | `services/support-admin-svc/astro/hr-portal` | Apply, track status, e-sign contract |
| HR management console (Django) | `services/support-admin-svc/django/hr_admin` | Shortlist, create contracts, manage roles |
| Payroll dashboard (LiveView) | `services/support-admin-svc/ex/payroll_web` | Create runs, monitor M-Pesa disbursement live |
| Statutory returns console (Django) | `services/support-admin-svc/django/hr_admin` | Submit and track KRA/NSSF/NHIF returns |

Languages: EN + SW (applicant portal). WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **M38 gate**: a Postgres constraint on `staff_contracts` enforces `m38_cert_id IS NOT NULL`. The application layer additionally verifies the cert event signature before inserting.
- **Payroll pre-flight**: the run engine checks that every `PayrollLine` maps to a signed `StaffContract` with a valid M38 cert; the run fails if any orphan lines are detected.
- **4-hour SLA monitoring**: Prometheus alert `eel_hr_payroll_disbursement_completion_ratio < 1.0` fires 3.5 hours into a run; PagerDuty escalates to payroll officer.
- **M-Pesa result reconciliation**: a nightly job compares `PayrollLine.mpesa_transaction_id` against the M-Pesa API statement; discrepancies are written to `eel_hr_mpesa_discrepancies` and escalated.
- **HSM-signed payroll approval**: commissioner HSM signature over `SHA-256(run_id || run_date || total_gross_kes || staff_count)` is stored in `PayrollRun`; any post-approval tampering is detectable.
- **Audit shadow**: every HR aggregate mutation is logged to `eel_admin_audit_shadow`.

## 8. Kenya statute alignment

- **Employment Act 2007 §45 (fair procedure)**: shortlisting decisions record `basis` and `score`; rejected applicants can request the decision record via a Data Subject Access Request.
- **Income Tax Act Cap 470 (PAYE)**: tax computation uses the KRA graduated table; the table version is pinned in `services/support-admin-svc/config/kra_paye_table.toml` and tagged per tax year.
- **NSSF Act 2013 (contributions)**: NSSF Tier I + Tier II contributions computed per the 2024 gazette rates.
- **NHIF Act Cap 255 (contributions)**: NHIF contributions computed from the NHIF contribution schedule pinned in config.
- **Kenya Data Protection Act §26 (minimisation)**: national ID stored as SHA-256 hash in `JobApplication`; only the M-Pesa number (needed for disbursement) is stored in clear text in `StaffContract`, encrypted at rest.

## 9. Honest caveats + external prerequisites

- M-Pesa B2C batch at 250k transactions depends on Safaricom granting the Commission a B2C shortcode with adequate throughput. The 4-hour target assumes Daraja API throughput >= 20 TPS per worker; actual throughput must be tested in a pre-election load test.
- KRA iTax integration requires the Commission to complete the KRA Tier-B MoU (WP-4.12). The adapter is ready; the iTax endpoint credentials are provisioned per MoU.
- NSSF and NHIF portal integrations use screen-scraping adapters where no official API exists; these are fragile and may break if the portals are updated.
- The M38 training gate applies only to contracted polling staff. Seconded civil servants from other ministries use a separate fast-track path (SpiceDB role `SECONDMENT_OFFICER`) that skips the M38 gate but requires a ministry letter uploaded to `staff_contracts.secondment_letter_url`.
- Payroll for permanent the Commission staff is out of scope; M55 covers only election-cycle temporary staff.

## 10. Test coverage + operational runbooks

Coverage under `services/support-admin-svc/crates/hr/tests/`: PAYE computation (table-driven against KRA gazette), M38 gate enforcement (integration, testcontainers), M-Pesa B2C batch round-trip (pytest-httpserver mock of Daraja API), 4-hour SLA regression (load test in `tests/load/hr-payroll-250k.k6.js`), statutory return format (golden-file against 2024 KRA P9A schema).

Runbooks under `docs/runbooks/`:

- `support-admin-svc.md` (section: hr-payroll) -- deploy, rollback, payroll run drain.
- `mpesa-b2c-failure-reconciliation.md` -- manual reconciliation when M-Pesa results are missing.
- `kra-return-deadline-escalation.md` -- escalation path when iTax submission fails before deadline.

Related documents: full spec at `docs/plans/modules/hr-payroll.md`; service capability catalogue at `services/support-admin-svc/docs/CAPABILITIES.md`; M38 interface at `docs/features/poll-worker-training-lms.md`.

### Payroll computation summary

A `PayrollLine` for a poll worker is computed as:

```
gross_kes        = StaffContract.gross_pay_kes
paye_kes         = f(gross_kes, kra_paye_table)   # KRA graduated table v2024-07
nssf_kes         = min(gross_kes * 0.06, 2160)    # Tier I + II cap per NSSF Act 2013
nhif_kes         = g(gross_kes, nhif_schedule)     # NHIF schedule per gazette
net_kes          = gross_kes - paye_kes - nssf_kes - nhif_kes
```

All deduction tables are versioned TOML files under `services/support-admin-svc/config/`; updating a table creates a new semantic version and all existing `PayrollRun` records retain a reference to the table version used.

### Key metrics and SLOs

| Metric | Target | Alert threshold |
|---|---|---|
| Payroll disbursement completion rate | 100% within 4 h | < 95% at 3.5 h: PagerDuty |
| M-Pesa B2C failure rate | < 0.5% | > 1%: manual reconciliation alert |
| M38 gate violation rate | 0% | Any: immediate build failure |
| KRA return on-time submission | 100% | Late: escalate to tax officer |
