# Finance Admin

Django admin console for election budget and expenditure management. Manages budgets, procurement records, payment vouchers, Controller of Budget (CoB) reports, and campaign finance disclosures. Integrates with IFMIS and CBK/FRC.

## Overview

Module: M54 (Budget Expenditure), M22 (Campaign Finance)

Finance operations back-office for tracking election spending, approving payment vouchers for supplier payments, managing campaign finance disclosures, and coordinating with IFMIS for government budget integration.

## Key Features

- **Budget Management**: Create and monitor election budgets by cost center and line item
- **Procurement Records**: Track purchase orders and supplier contracts
- **Payment Vouchers**: Submit, approve, and track payment vouchers to IFMIS
- **Utilization Tracking**: Monitor actual spending vs. budget allocations
- **Campaign Finance Disclosures**: Record candidate/party financial disclosures with cash aggregates
- **CoB Reporting**: Generate Controller of Budget compliance reports
- **FRC Referral**: Flag suspicious transactions for FRC (Financial Reporting Centre) investigation

## HTMX Endpoints

- POST `/finance/admin/voucher/{id}/approve` - Approve payment voucher and submit to IFMIS
- GET `/finance/admin/budget/{id}/utilisation` - View spending progress vs. budget

## Models (Managed=False)

- Budget - Election budget allocations
- Procurement - Purchase orders and contracts
- PaymentVoucher - Payment requests to suppliers
- CoBReport - Controller of Budget compliance reports (read-only)
- CandidateDisclosure - Financial disclosure records
- CashAggregate - Suspicious cash flow aggregates (read-only from analysis)

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `IFMIS_ADAPTER_URL` - IFMIS voucher submission endpoint
- `CBK_FRC_ADAPTER_URL` - CBK Financial Reporting Centre referral

## Running Locally

```bash
uv sync
uv run manage.py runserver
```

## Tests

```bash
uv run pytest -vxs tests/ci
```

## Deployment

WCAG 2.2 AA compliant. Pod resource: CPU 500m, Memory 512Mi.

Kubernetes manifest: `services/support-admin-svc/deploy/base/finance-deployment.yaml`

Production run:

```bash
gunicorn finance_admin.wsgi:application
```

## Permissions

FINANCE_OFFICER, FINANCE_SUPERVISOR, COB_LIAISON roles via SpiceDB. Supervisors approve vouchers; officers submit; COB liaison exports compliance reports.

## Notes

- Payment vouchers integrate with IFMIS for government accounting
- Campaign finance disclosures are public records (delay-published)
- FRC referrals are confidential and reviewed before transmission
