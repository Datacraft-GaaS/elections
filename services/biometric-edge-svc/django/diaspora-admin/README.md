# Diaspora Admin

Django admin console for diaspora voting operations. Manages embassy station configuration, diaspora voter roll extracts, Form 34AD (diaspora result declaration), and pouch custody event audit trails.

## Overview

Module: M24 (Diaspora Voting)

Back-office tool for diaspora election coordinators to manage voting infrastructure at embassies and overseas locations, track ballot custody, and oversee result declarations from diaspora stations.

## Key Features

- **Embassy Station Management**: Configure polling stations at embassies with voter roll extracts
- **Voter Roll Extract**: Diaspora-eligible voter subset, versioned and audited
- **Form 34AD Audit Trail**: Diaspora result declarations with custody chain verification
- **Pouch Custody Log**: Real-time custody event timeline from diplomatic couriers
- **Admin Views**: EmbassyStationAdmin, DiasporaRollAdmin, PouchCustodyAdmin

## HTMX Endpoints

- GET `/diaspora/admin/station/{id}/roll` - Voter roll extract for station
- GET `/diaspora/admin/pouch/{id}/events` - Custody event timeline with seal verification

## Models (Managed=False)

- EmbassyStation - Overseas polling locations
- DiasporaRoll - Voter subsets extracted per election
- Form34AD - Diaspora result declarations (read-only)
- PouchCustodyEvent - Ballot pouch chain of custody (read-only)

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection to biometric-edge-svc database
- `EDGE_API_URL` - Biometric edge service API endpoint
- `DIASPORA_ADMIN_ALLOWED_GROUPS` - Comma-separated SpiceDB roles

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

Kubernetes manifest: `services/biometric-edge-svc/deploy/base/diaspora-admin-deployment.yaml`

Production run:

```bash
gunicorn diaspora_admin.wsgi:application
```

## Permissions

DIASPORA_ADMIN, DIASPORA_OPERATOR roles enforced via SpiceDB. Read-only access to Form 34AD and custody events.

## Notes

- Diaspora voter roll is a subset of BVR (Voter Registration) database
- Pouch custody events are immutable and digitally signed
- Embassy station configuration requires coordination with Ministry of Foreign Affairs
