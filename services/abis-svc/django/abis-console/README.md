# ABIS Console

Django admin console for biometric matching operators. Manages adjudication queue, enclave attestation records, and match request audits in a secure, role-based back-office tool.

## Overview

Module: M03 (ABIS - Automated Biometric Identification System)

Internal tool for ABIS operators and supervisors to adjudicate biometric match disputes, verify HSM enclave attestations, and audit all matching operations.

## Key Features

- **Adjudication Queue**: Review disputed biometric matches with side-by-side template comparison
- **Inline Actions**: Approve, reject, or escalate matches to supervisor review
- **Enclave Attestation**: View HSM-signed attestation records for compliance audits
- **Match Audit Log**: Read-only MatchRequest audit trail with timestamp and operator details
- **Admin Views**: AdjudicationCaseAdmin (with inline action buttons), EnclaveAttestAdmin

## HTMX Endpoints

- POST `/abis/console/adjudicate/{id}` - Submit adjudication decision
- GET `/abis/console/adjudicate/{id}` - Render adjudication form
- POST `/abis/console/attest/{id}` - View/download attestation proof

## Models (Managed=False)

- AdjudicationCase - Disputed match cases with decision state
- EnclaveAttestation - HSM attestation proofs (read-only)
- MatchRequest - Biometric match audit (read-only)

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `ABIS_API_URL` - ABIS matcher gRPC endpoint
- `SPICEDB_ENDPOINT` - SpiceDB authorization service

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

WCAG 2.2 AA compliant. Pod resource: CPU 250m, Memory 256Mi.

Kubernetes manifest: `services/abis-svc/deploy/base/console-deployment.yaml`

Production run:

```bash
gunicorn abis_console.wsgi:application
```

## Permissions

Requires ABIS_OPERATOR or ABIS_SUPERVISOR role via SpiceDB. Supervisors can escalate; operators adjudicate their assigned queue.

## Notes

- Adjudication decisions are immutable (audit compliance)
- Attestation download requires supervisor approval
- Queue auto-assigns based on operator load
