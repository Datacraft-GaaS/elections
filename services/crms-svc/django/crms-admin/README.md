# CRMS Admin

Django admin console for candidate and party registration. Manages candidate nominations, clearance decisions, ballot proof generation, political party registrations, primary oversight, nomination fees, and party symbol/colour registry.

## Overview

Module: M12 (CRMS - Candidate & Party Registry), M45 (Party Registration), M46 (Party Primaries), M47 (Nomination Fee Payment)

Back-office tool for electoral commission staff and legal teams to manage candidate/party registration workflows, enforce eligibility clearance rules, and oversee primary elections.

## Key Features

- **Nomination Management**: Record, review, and approve candidate nominations with eligibility checks
- **Clearance Decisions**: Multi-step approval workflow (submitted -> checked -> approved/rejected)
- **Ballot Proof Generation**: Generate sample ballots showing candidate placement and symbols
- **Party Registration**: Manage political party registrations and renewals
- **Primary Oversight**: Enroll parties in primaries, track primary election results
- **Symbol Registry**: Register and assign ballot symbols and colors
- **Fee Payment Integration**: Validate nomination fee payments via M-Pesa

## HTMX Endpoints

- POST `/crms/admin/nomination/{id}/clearance` - Review clearance checklist and approve/reject
- POST `/crms/admin/symbol/{id}/approve` - Approve party symbol/color choice

## Models (Managed=False)

- Candidate - Nomination records
- Nomination - Candidate ballot submissions
- ClearanceDecision - Eligibility approval log
- BallotProof - Sample ballots for preview
- PoliticalParty - Party registrations
- PrimaryOptIn - Party primary enrollment
- PrimaryResult - Primary election tallies
- SymbolSubmission, ColourRegistration - Ballot symbol registry
- BallotDesignPackage - Ballot specification packages

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `CRMS_API_URL` - CRMS service API endpoint
- `KRA_ITAX_ADAPTER_URL` - KRA tax compliance verification
- `EACC_ADAPTER_URL` - EACC corruption index check

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

Kubernetes manifest: `services/crms-svc/deploy/base/admin-deployment.yaml`

Production run:

```bash
gunicorn crms_admin.wsgi:application
```

## Permissions

CRMS_OFFICER, CRMS_SUPERVISOR, CRMS_AUDITOR roles via SpiceDB. Supervisors sign off clearance; auditors review compliance.

## Notes

- Nomination deadlines are enforced via election calendar
- Clearance checks integrate external systems (KRA, EACC)
- Ballot symbols must be unique per party per election
