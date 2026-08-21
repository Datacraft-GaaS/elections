# Accreditation Admin

Django admin console for media and observer accreditation. Manages media organisation and individual journalist accreditation, observer organisation and individual accreditation, badge generation, and QR code signing.

## Overview

Module: M42 (Media Accreditation), M43 (Observer Accreditation)

Back-office tool for processing media and observer accreditation applications, verifying eligibility, generating tamper-proof credentials, and managing badge distribution.

## Key Features

- **Media Accreditation**: Process journalist and media organization accreditation applications
- **Observer Accreditation**: Manage domestic and international observer registration and vetting
- **Application Review**: Multi-step approval workflow with document verification
- **Badge Generation**: Produce laminated or digital accreditation badges with QR codes
- **QR Code Signing**: Cryptographically sign badge QR codes for verification at polling stations
- **Admin Views**: MediaAccredAdmin (approve/reject), ObserverAccredAdmin, BadgeGenerationAdmin

## HTMX Endpoints

- POST `/accred/admin/media/{id}/approve` - Approve or reject media accreditation
- POST `/accred/admin/observer/{id}/badge` - Generate and download accreditation badge

## Models (Managed=False)

- MediaAccreditation - Individual journalist accreditations
- MediaOrganisation - News outlet registrations
- ObserverOrganisation - Domestic/international observer group registrations
- IndividualObserver - Individual observer accreditation records

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `MCK_ADAPTER_URL` - Media Council of Kenya verification
- `BADGE_SIGN_KEY` - Private key for QR code signing (PEM format)

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

Kubernetes manifest: `services/training-svc/deploy/base/accreditation-deployment.yaml`

Production run:

```bash
gunicorn accreditation.wsgi:application
```

## Permissions

ACCRED_OFFICER, ACCRED_SUPERVISOR roles via SpiceDB. Officers process applications; supervisors approve badge issuance.

## Notes

- Media accreditation requires MCK verification (Kenya media regulator)
- Observer accreditations are vetted against criminal/integrity databases
- QR codes are valid only during election period
- Badge generation is one-time (no re-issuance without reapplication)
