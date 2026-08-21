# Support Admin

Django admin console for special voter support and long-term archive management. Manages special voting facilities (KPS/KDF/UNHCR), pre-mark validation events, spoilt ballot adjudication, archive ingest jobs, and digital preservation attestations.

## Overview

Module: M25 (Special Voter Workflows), M27 (Intent Preservation), M69 (Long-Term Archive)

Support operations back-office for managing accessibility accommodations for special voters, overseeing election artifact preservation, and managing post-election archive ingestion into national digital repository.

## Key Features

- **Special Voter Facilities**: Manage election centers for military (KDF), police (KPS), refugees (UNHCR)
- **Special Voter Events**: Log voting assistance events with privacy protections
- **IDP Reassignment**: Manage internally displaced persons returning to original polling stations
- **Pre-Mark Validation**: Review and approve pre-marked ballots for voters needing assistance
- **Spoilt Ballot Adjudication**: Approve or reject spoilt ballot claims with evidence review
- **Archive Ingest**: Receive election artifacts for long-term preservation (results, forms, audit logs)
- **Preservation Attestations**: Digital signatures confirming archive integrity and provenance

## HTMX Endpoints

- GET `/support/admin/special-voter/{id}/events` - Timeline of voting assistance events
- POST `/support/admin/archive/{id}/ingest` - Submit artifacts for archival ingest

## Models (Managed=False)

- SpecialVoterFacility - KPS/KDF/UNHCR voting centers
- SpecialVoterEvent - Voting assistance events (read-only, privacy-protected)
- IDPReassignment - IDP status tracking
- PreMarkValidationEvent - Pre-marked ballot approvals
- SpoiltBallotEvent - Spoilt ballot audit trail (read-only)
- ArchiveCollection - Archive groupings by election/type
- IngestJob - Archive ingest job status tracking

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `KPS_ADAPTER_URL` - Kenya Police Service coordination
- `KNADS_ADAPTER_URL` - KNADS national digital archive submission
- `DATACITE_ADAPTER_URL` - DataCite DOI registration for archived materials

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

Kubernetes manifest: `services/support-admin-svc/deploy/base/support-admin-deployment.yaml`

Production run:

```bash
gunicorn support_admin.wsgi:application
```

## Permissions

SUPPORT_OFFICER, ARCHIVE_OFFICER, SPECIAL_VOTER_MANAGER roles via SpiceDB. Support officers log events; archive officers manage ingest; special voter manager oversees facilities.

## Notes

- Special voter event records are encrypted and access-controlled (privacy)
- Spoilt ballot adjudication is final (no appeal after adjudication decision)
- Archive ingest generates DOIs for long-term citability
- Preservation attestations use NIST guidelines for digital archiving
