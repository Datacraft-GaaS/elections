# Secrecy Assurance Red Team

Secrecy assurance red-team service for adversarial testing of ballot secrecy enforcement. Orchestrates red-team engagements against staging environment clones, records findings, and produces IAB attestations.

## Overview

Module: M61 (Secrecy Assurance Testing)

Independent security testing service that attempts to break ballot secrecy protections, identify vulnerabilities, and document them for remediation. Operates only on staging copies (isolation mode enforced).

## FastAPI Endpoints

- POST `/engagements` - Create red-team engagement
- GET `/engagements/{id}` - Retrieve engagement details
- POST `/engagements/{id}/findings` - Log security finding
- GET `/health` - Service health check

## Pydantic Models

- `RedTeamEngagement` - Engagement record (id, scope, start_ts, end_ts, testers: list)
- `Finding` - Vulnerability finding (engagement_id, severity, description, remediation, evidence)
- `IABAttestation` - Signed attestation (engagement_id, attester_id, summary_hash, signature)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `STAGING_CLONE_URL` - Staging environment URL for testing
- `IAB_API_URL` - IAB (Independent Audit Board) for attestation
- `ISOLATION_MODE` - MUST be 'staging' (prevents accidental prod connection)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn secrecy_redteam.wsgi:application --workers 2
```

## Data Isolation

- `ISOLATION_MODE='staging'` guard prevents accidental connection to production systems
- Tests run only against staging environment clones
- No production data is accessed or modified
- Findings are reviewed by IAB before public disclosure

## Notes

- Red-team scope includes biometric template extraction, ballot linkage, and result inference
- High-severity findings (complete secrecy breach) trigger emergency remediation
- Each engagement is independently attested by IAB members
- Findings are used for system hardening before production elections
