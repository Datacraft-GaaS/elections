# payroll

HR and payroll crate for M55. Manages job applications, staff contracts, payroll runs for ~250,000 temporary election staff, and statutory deductions (KRA iTax, NSSF, NHIF/SHIF), with M-Pesa B2C disbursement.

## What it does

Election-day operations require ~250,000 temporary staff contracted per cycle. The payroll crate tracks: job applications and vetting (linked to M38 LMS certification), contract terms and assignment data (linked to M33 personnel dispatch), gross pay calculation, statutory deductions (PAYE via KRA iTax API, NSSF, NHIF/SHIF), and net pay disbursement via M-Pesa B2C. A full payroll run for 250,000 records must complete within 4 hours. Strict SoD controls separate HR data entry, payroll approval, and disbursement authorisation.

## Public API surface

- `mod staff` - `JobApplication`, `StaffContract`, `onboard_staff()`, `terminate_contract()`
- `mod payroll_run` - `PayrollRun`, `PayrollRunStatus`, `start_run()`, `approve_run()`, `disburse_run()`
- `mod deductions` - `StatutoryDeduction`, `paye_compute()`, `nssf_compute()`, `nhif_compute()`
- `mod mpesa` - `DisbursementBatch`, `submit_b2c_batch()`, `handle_b2c_callback()`
- `mod kra` - KRA iTax P10/P9A file generation and submission

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `mpesa-live` | off | Live M-Pesa Daraja B2C API (requires sandbox/prod keys) |
| `kra-live` | off | Live KRA iTax API submission |
| `nssf-live` | off | Live NSSF API submission |

## Test commands

```bash
cargo nextest run -p payroll
cargo nextest run -p payroll --test integration -- --include-tag payroll
# Load test (250k scale)
k6 run services/support-admin-svc/loadtest/payroll-250k.js
```

## Dependencies of note

- `eel-auth-rs` (workspace): SoD role checks
- `eel-outbox-rs` (workspace): disbursement event publishing
- `eel-resilience-rs` (workspace): circuit breaker for M-Pesa B2C
- `services/support-admin-svc/py/mpesa-b2c/` - Python B2C batch adapter

## Related docs

- `docs/plans/modules/M55-hr-payroll.md`
- `packages/openapi-contracts/support-admin/payroll.yaml`
- `docs/runbooks/support-admin-svc-payroll-incident.md`
