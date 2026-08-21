# Declaration Admin

Django admin console for election result declaration and publication. Manages Form 34A/B/C result forms, declaration signing workflow, petition window tracking, and ballot serial chain reconciliation.

## Overview

Module: M10 (RTS Backend), M51 (Result Declaration), M63 (Ballot Serial Chain)

Back-office tool for returning officers and election management staff to sign results declarations, manage publication workflow, and ensure ballot serial chain integrity across all polling stations.

## Key Features

- **Form Management**: Upload and review Form 34A (presidential/parliamentary tallies), 34B (county tallies), 34C (supplementary)
- **Declaration Signing**: Sign results with HSM-based digital signature before publication
- **Publication Workflow**: Manage public announcement of results with petition window tracking
- **Merkle Root Management**: Publish cryptographic roots of result set for integrity verification
- **Serial Chain Reconciliation**: Verify ballot serial assignments match physical inventory
- **Certificate of Election**: Generate and sign official election certificates
- **Audit Snapshots**: Versioned snapshots of results at each publication stage

## HTMX Endpoints

- POST `/rts/admin/declaration/{id}/sign` - Sign results with HSM certificate
- POST `/rts/admin/declaration/{id}/publish` - Publish results and trigger notification events

## Models (Managed=False)

- Form34A, Form34B, Form34C - Result form records
- PublicationSnapshot - Versioned result snapshots
- MerkleRoot - Cryptographic roots (read-only, managed by RTS service)
- Declaration - Result declaration metadata
- CertificateOfElection - Official election certificates
- SerialAssignment - Ballot serial chain mapping (read-only)

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `RTS_API_URL` - RTS service API endpoint
- `HSM_ENDPOINT` - HSM (Hardware Security Module) for result signing

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

Kubernetes manifest: `services/rts-svc/deploy/base/declaration-admin-deployment.yaml`

Production run:

```bash
gunicorn declaration_admin.wsgi:application
```

## Permissions

RTS_OFFICER, RETURNING_OFFICER, RTS_AUDITOR roles via SpiceDB. Returning officers sign declarations; auditors verify chain integrity.

## Notes

- Result declarations are immutable once signed
- Petition window is legally defined per election (typically 14 days post-publication)
- Ballot serial chain reconciliation prevents result inflation fraud
- HSM signing requires valid credentials and takes several seconds
