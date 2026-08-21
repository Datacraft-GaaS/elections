# budget

Budget and expenditure crate for M54. Manages the election budget ledger, procurement records, payment vouchers, and Consolidated Budget Report (CoB), with IFMIS and PPRA integration.

## What it does

Tracks every financial commitment and disbursement against the election budget. Procurement follows PPRA rules; vouchers route through IFMIS for government-side posting. The module enforces segregation-of-duty (SoD) controls: commitment, approval, and disbursement roles are distinct SpiceDB permissions. Financial forensics (M66, governance-svc) consumes read-only projections from this crate.

## Public API surface

- `mod budget_line` - `BudgetLine`, `Commitment`, `create_commitment()`
- `mod voucher` - `PaymentVoucher`, `VoucherStatus`, `approve_voucher()`, `submit_to_ifmis()`
- `mod procurement` - `ProcurementRecord`, `ppra_reference_check()`
- `mod cob_report` - `CoBReport`, `generate_cob_report()`
- `mod projections` - Read-model views for forensics and OAG export

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `ifmis-live` | off | Live IFMIS API posting (requires MoF credentials) |
| `ppra-live` | off | Live PPRA contract reference lookup |

## Test commands

```bash
cargo nextest run -p budget
cargo nextest run -p budget --test integration -- --include-tag budget
```

## Dependencies of note

- `eel-auth-rs` (workspace): SoD permission checks via SpiceDB
- `eel-outbox-rs` (workspace): voucher event publishing
- `eel-pii-rs` (workspace): financial PII classification
- `services/support-admin-svc/py/ifmis/` - Python IFMIS adapter

## Related docs

- `docs/plans/modules/M54-budget-expenditure.md`
- `packages/openapi-contracts/support-admin/budget.yaml`
- `packages/event-schemas/support/budget.*.json`
