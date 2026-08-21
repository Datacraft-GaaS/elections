# Governance Admin

Multi-module governance back-office for election audits, election court interface, post-election review, recall/referendum mechanics, budget/expenditure, financial forensics, OAG liaison, and parliamentary reporting.

## Overview

Module: M21 (RLA Audit), M40 (Election Court), M41 (Post-Election Review), M48 (Recall), M49 (Referendum), M50 (By-Election), M54 (Budget), M56 (County Coordination), M57 (International Observer Coordination), M58 (Parliamentary/Judiciary Reporting), M59 (OAG Interface), M62 (Transmission Legal Audit), M66 (Financial Forensics)

Centralized governance tool for legal, audit, budget, and international coordination functions spanning the entire election lifecycle.

## Key Features

- **Audit Run Management**: Create, monitor, and close RLA (Risk-Limiting Audit) engagements
- **Evidence Bundles**: Collect audit evidence and attestations, export audit packages
- **Recall/Referendum Workflow**: Process recall petitions, manage referendum mechanics
- **Budget Tracking**: Manage election budgets, track expenditures, reconcile with IFMIS
- **Financial Forensics**: View anomaly detection results and referral packages for CBK/FRC
- **Election Court Interface**: File election petitions, track court rulings
- **OAG Liaison**: Submit reports to OAG for constitutional review

## HTMX Endpoints

- GET `/governance/admin/audit/{id}/export` - Download audit evidence bundle
- POST `/governance/admin/recall/{id}/verify` - Verify petition signatures and declare recall

## Models (Managed=False)

- AuditRun - RLA engagement records
- BundleItem - Audit evidence items
- EvidenceBundle - Packaged evidence for court/OAG
- RecallPetition - Voter-initiated recall petitions
- Referendum - Referendum ballot initiatives
- Budget - Election budget allocations

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `GOVERNANCE_API_URL` - Governance service API endpoint
- `ECOURT_ADAPTER_URL` - Election Court filing system
- `IFMIS_ADAPTER_URL` - IFMIS budget tracking

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

Kubernetes manifest: `services/governance-svc/deploy/base/admin-deployment.yaml`

Production run:

```bash
gunicorn governance_admin.wsgi:application
```

## Permissions

GOVERNANCE_OFFICER, LEGAL_TEAM, AUDITOR, OAG_LIAISON roles via SpiceDB. Legal team approves court filings; auditors manage audit runs.

## Notes

- Recall petitions require signature verification before proceeding
- Budget reconciliation must complete before final election certificate issuance
- Financial forensics referrals are reviewed by compliance team before sending to CBK/FRC
