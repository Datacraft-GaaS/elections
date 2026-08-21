# AI Monitoring

Model observability and registry Python service. Tracks AI model decisions, PII stripping events, model card metadata, and model lifecycle. FastAPI registry API plus Kafka consumer for event ingestion.

## Overview

Module: M28 (AI Monitoring Stack)

Central observability platform for all AI/ML systems in the election ecosystem. Logs model inferences, tracks PII removal events, maintains model registry with versioning, and supports compliance audits.

## FastAPI Endpoints

- POST `/models` - Register new AI model with metadata
- GET `/models/{id}` - Retrieve model details
- POST `/models/{id}/decisions` - Log model decision (inference event)
- GET `/health` - Service health check

## Pydantic Models

- `AIModel` - Model registry (id, name, version, risk_tier, intended_use)
- `AIDecisionLog` - Inference event (model_id, input_hash, output_hash, pii_stripped, timestamp)
- `ModelCard` - Model documentation (id, intended_use, limitations, fairness_metrics)
- `PIIStrippingEvent` - PII removal tracking (model_id, original_pii_type, timestamp)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints (comma-separated)
- `KAFKA_CONSUMER_GROUP` - Consumer group for event ingestion
- `LITELLM_ADAPTER_URL` - LiteLLM API gateway for LLM-based models
- `MODEL_REGISTRY_S3_BUCKET` - S3 bucket for model artifacts

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn ai_monitoring.wsgi:application --workers 4
```

## Notes

- PII stripping is mandatory before model decision logging
- Model cards document fairness metrics and risk tiers
- All model inferences are logged for compliance audits
- Risk tier determines audit frequency (HIGH: every inference, LOW: sampled)
