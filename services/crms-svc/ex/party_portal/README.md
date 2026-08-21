# party_portal (Phoenix app)

Phoenix LiveView portal for political party officials to manage registrations, primary results, and nomination fees.

## Module

M12 (crms), M45 (party-registration), M46 (party-primaries-oversight), M47 (nomination-fee-payment) | crms-svc | WP-4.9

## Purpose

Party-facing Phoenix LiveView application. Allows authorised party officials to:
submit and track party registration, submit primary results, pay candidate nomination
fees via M-Pesa (STK push or C2B paybill), view compliance status (clearance certs
from EACC, NCIC, KRA). Integrated with Daraja C2B callback via Broadway consumer.

## LiveViews

| Module | Description |
|---|---|
| `PartyDashboardLive` | Party overview: registration status, compliance |
| `RegistrationLive` | Party registration form and document upload |
| `PrimaryResultsLive` | Primary election results entry and submission |
| `NominationFeePaymentLive` | STK push initiation and payment status polling |
| `SymbolRegistrationLive` | Party symbol and colour submission (Pantone code) |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `MpesaCallbackConsumer` | `eel.payments.mpesa-c2b` | Confirm nomination fee payments |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:fee_reconciliation` | `FeeReconciliationWorker` | Daily payment reconciliation |
| `:symbol_approval` | `SymbolApprovalWorker` | Notify party of symbol approval |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/crms-svc/ex/party_portal
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `MPESA_SHORTCODE` | Daraja shortcode for nomination fees |
| `CRMS_API_URL` | crms-svc Rust API |
| `PANTONE_ADAPTER_URL` | Pantone colour validation |

## WCAG

2.2 AA.

## Deployment Notes

- Kubernetes: crms-svc ingress (public, TLS)
- K8s manifest: services/crms-svc/deploy/base/party-portal-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M12, M45, M46, M47)
- crms-svc/py/daraja (M-Pesa callback handler)
- integrations/mpesa-daraja
- integrations/pantone
