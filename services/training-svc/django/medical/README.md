# Medical First Response Admin

Django admin console for medical incident management during elections. Manages incident reports at polling stations, dispatch requests to first responders (KRCS/MoH/NDMU), cluster alerts, and coordination protocols.

## Overview

Module: M60 (Medical First Response)

Emergency management system for election-day medical incidents at polling stations. No PHI (Protected Health Information) is stored; triage category and location only, to comply with HIPAA-equivalent constraints.

## Key Features

- **Incident Reporting**: Log medical incidents at polling stations (no PHI stored)
- **Triage Categorization**: Record triage category (non-emergency, urgent, critical) without patient details
- **Dispatch Requests**: Route incident notifications to available first responders
- **Cluster Alerts**: Trigger alerts if multiple incidents cluster in one area/time window
- **Coordination Protocol**: Coordinate with KRCS (Red Cross), MoH (Health), NDMU (National Disaster)
- **Admin Views**: MedicalIncidentAdmin, DispatchAdmin (approve/dispatch), ClusterAlertAdmin

## HTMX Endpoints

- POST `/medical/admin/incident/{id}/dispatch` - Submit dispatch request to first responders
- GET `/medical/admin/cluster/{id}/alert` - View incident cluster alert details

## Models (Managed=False)

- MedicalIncident - Incident records (triage category + location only)
- DispatchRequest - First responder dispatch records
- ClusterAlert - Geographic/temporal incident cluster flags

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `ECC_API_URL` - Election Command Centre incident coordination
- `KRCS_WEBHOOK_URL` - Kenya Red Cross dispatch webhook

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

Kubernetes manifest: `services/training-svc/deploy/base/medical-deployment.yaml`

Production run:

```bash
gunicorn medical.wsgi:application
```

## Permissions

MEDICAL_COORDINATOR, MEDICAL_SUPERVISOR roles via SpiceDB. Coordinators log incidents; supervisors approve dispatch.

## Data Privacy

No PHI stored per HIPAA-equivalent constraint. Only triage category (non-emergency/urgent/critical) and station location recorded. Patient identity is never recorded.

## Notes

- All dispatch requests go directly to first responder services (KRCS/MoH/NDMU)
- Cluster alerts notify supervisors if 3+ incidents occur in same constituency within 2 hours
- Incidents are logged in real-time from mobile field apps or manual station reports
- No incident records are shared with public (internal only)
