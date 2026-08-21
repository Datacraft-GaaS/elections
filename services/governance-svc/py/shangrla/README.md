# SHANGRLA RLA Algorithm

SHANGRLA (SuperFastHunting And amuNuRlation GeneraLs Audit) risk-limiting audit algorithm implementation. Deterministic Python library called by the rla-desktop Tauri app via gRPC wrapper for ballot-level audits.

## Overview

Module: M21 (RLA Audit Tool)

Python implementation of Stark's SHANGRLA algorithm for statistical ballot audits. Determines minimum ballot sample size and whether audited samples confirm election outcome.

## FastAPI Endpoints

- POST `/rla/sample` - Generate deterministic ballot sample given entropy
- POST `/rla/verify` - Verify sample results against outcome claim
- GET `/health` - Service health check

## Pydantic Models

- `AuditRun` - Audit job (id, election_id, risk_limit: float, entropy_source, created_at)
- `SampledBallot` - Audit sample (run_id, ballot_id, expected_outcome, audited_outcome, auditor_id)
- `AuditResult` - Audit conclusion (run_id, risk_achieved: float, passed: bool, audit_status)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `NIST_BEACON_URL` - NIST Randomness Beacon for entropy (default: nist.gov)
- `RLA_RISK_LIMIT_DEFAULT` - Default risk limit (default: 0.05)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn shangrla.wsgi:application --workers 2
```

## Determinism

- Deterministic given same entropy source + ballot manifest
- Supports NIST Beacon entropy or custom entropy seed
- Reproducible audit samples for transparency

## Risk Limit Interpretation

- Risk limit = 0.05 (5%): 95% confidence in outcome
- Outcome is confirmed if risk achieved <= risk limit after all samples audited
- If risk not confirmed, expand sample and repeat

## Notes

- Requires complete ballot manifest (all ballot IDs with candidate marks)
- Sample size grows with risk limit (5% = ~500 ballots, 1% = ~2000 ballots)
- Audit assumes no invalid ballots in sample
- Works for any election type (presidential, parliamentary, county)
