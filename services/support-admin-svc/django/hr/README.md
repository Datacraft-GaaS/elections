# HR Admin

Django admin console for human resources and payroll management. Manages job applications, staff contracts, payroll runs, statutory deductions (PAYE/NSSF/NHIF/SHIF), attendance records, and triggers M-Pesa B2C payouts.

## Overview

Module: M55 (HR Payroll), M33 (Mobile Personnel Dispatch), M38 (Poll Worker Training LMS), M60 (Medical First Response)

HR operations back-office for managing election staff hiring, contract administration, payroll processing, statutory deduction calculations, and mobile workforce dispatch coordination.

## Key Features

- **Job Applications**: Process applications for poll workers, election staff, and support roles
- **Staff Contracts**: Create, manage, and archive employment contracts
- **Payroll Runs**: Execute monthly payroll runs with batch approval workflow
- **Statutory Deductions**: Calculate PAYE (tax), NSSF (pension), NHIF/SHIF (health insurance)
- **Attendance Tracking**: Record presence for field staff (manually or from field mobile app)
- **B2C Payout**: Trigger M-Pesa bulk transfers once payroll approved (250k records SLA: 4 hours)
- **Personnel Dispatch**: Coordinate field staff assignments to polling stations

## HTMX Endpoints

- POST `/hr/admin/payroll/{id}/approve` - Approve payroll batch and trigger M-Pesa payout
- GET `/hr/admin/staff/{id}/contract` - View/download staff contract

## Models (Managed=False)

- JobApplication - Staff hiring applications
- StaffContract - Employment agreements
- PayrollRun - Monthly payroll batches
- StatutoryDeduction - Tax and benefit calculations
- PersonnelAttendanceRecord - Field staff presence (read-only from mobile)
- MedicalIncident - Emergency medical events (read-only)

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `MPESA_B2C_ADAPTER_URL` - M-Pesa bulk payout endpoint
- `NSSF_ADAPTER_URL` - NSSF pension contribution calculation
- `NHIF_ADAPTER_URL` - NHIF health insurance deduction
- `KRA_ITAX_ADAPTER_URL` - KRA tax calculation service

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

WCAG 2.2 AA compliant. Pod resource: CPU 1000m, Memory 1Gi.

Kubernetes manifest: `services/support-admin-svc/deploy/base/hr-deployment.yaml`

Production run:

```bash
gunicorn hr_admin.wsgi:application
```

## Permissions

HR_OFFICER, HR_SUPERVISOR, PAYROLL_APPROVER roles via SpiceDB. Officers process applications and contracts; supervisors and approvers handle payroll.

## Notes

- Payroll runs must complete within 4-hour SLA for 250k records
- M-Pesa daily limit (5000 txn/day default) must be increased with Safaricom
- PAYE calculations use current KRA tax bands (updated each tax year)
- Attendance records can be manually entered or synced from mobile apps
