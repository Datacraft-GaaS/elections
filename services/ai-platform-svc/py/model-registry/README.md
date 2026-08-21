# Model Registry

Dedicated model registry service for managing AI model versions, deployment approvals, and fairness attestations. Separate from ai-monitoring for independent scaling and dedicated model lifecycle management.

## Overview

Module: M28 (AI Monitoring Stack)

Centralized registry for all ML models deployed in election systems, with version control, deployment workflows, and fairness compliance tracking.

## FastAPI Endpoints

- GET `/models` - List all models
- POST `/models` - Register new model
- GET `/models/{id}/versions` - Model version history
- POST `/models/{id}/deploy` - Request deployment approval
- GET `/health` - Service health check

## Pydantic Models

- `ModelVersion` - Version record (model_id, version, artifact_hash, created_at, deployed_at)
- `DeploymentApproval` - Approval record (approver_id, model_id, version, attestation, approved_at)
- `FairnessAttestation` - Fairness metrics (model_id, version, bias_audit_url, disparate_impact_ratio)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `S3_BUCKET` - S3 bucket for model artifacts
- `S3_PREFIX` - S3 key prefix (default: models/)
- `IAB_API_URL` - IAB (Independent Audit Board) API for fairness attestation
- `AI_MONITORING_URL` - AI Monitoring service for decision logging

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn model_registry.wsgi:application --workers 2
```

## Notes

- Model deployment requires fairness audit completion (bias testing mandatory)
- Version numbers follow semantic versioning (major.minor.patch)
- Artifacts are stored in S3 with checksum validation
- Approvals require sign-off from IAB (Independent Audit Board)
