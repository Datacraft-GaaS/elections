# BVR Admin

Django admin console for voter registration management. Manages voter records, biometric templates, audit events, Merkle roots for integrity verification, CVR (Complete Voter Register) events, deregistration appeals, and voter card batches.

## Overview

Module: M02 (BVR Backend), M35 (CVR), M36 (Voter Deregistration Appeal)

Core back-office tool for registration staff to manage the voter register, approve/reject voter deregistration appeals, oversee voter card printing batches, and audit all registration changes.

## Key Features

- **Voter Record Management**: Create, edit, deactivate voters with complete address history
- **Template History**: View biometric template versions with enrollment timestamps
- **Audit Events**: Immutable audit log of all voter register changes (read-only)
- **Merkle Root Publication**: Publish integrity roots for CVR verification
- **Deregistration Appeals**: Approve/reject voter removal requests with supervisor escalation
- **Voter Card Batch Admin**: Manage printing batches, card serial ranges, reconciliation

## HTMX Endpoints

- GET `/bvr/admin/voter/{id}/history` - Voter change history and template versions
- POST `/bvr/admin/deregistration/{id}/review` - Submit appeal decision (approve/reject)

## Models (Managed=False)

- Voter - Versioned voter records
- BiometricTemplate - Fingerprint/iris templates
- AuditEvent - Registration change audit log (read-only)
- MerkleRoot - CVR integrity roots
- VoterRegisterEvent - CVR publication events
- DeregistrationCase - Voter removal appeals
- VoterCard - Printed voter ID cards
- CardBatch - Batch printing jobs

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `BVR_API_URL` - BVR service API endpoint
- `IPRS_ADAPTER_URL` - IPRS (Immigration database) verification service

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

Kubernetes manifest: `services/bvr-svc/deploy/base/admin-deployment.yaml`

Production run:

```bash
gunicorn bvr_admin.wsgi:application
```

## Permissions

BVR_REGISTRAR, BVR_SUPERVISOR, BVR_AUDITOR roles via SpiceDB. Supervisors approve deregistration appeals; auditors review changes only.

## Notes

- Voter deactivation is preferred over deletion for audit compliance
- Merkle roots are signed and published to immutable ledger
- Voter card batches trigger print-on-demand workflows
