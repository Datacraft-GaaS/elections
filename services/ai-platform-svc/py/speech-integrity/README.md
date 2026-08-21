# Speech Integrity

Speech integrity ML service for content classification and safety incident management. Classifies content flags (hate speech, disinformation, election interference), manages safety incidents, and packages evidence for NCIC (National Cohesion and Integration Commission).

## Overview

Module: M17 (Speech Integrity)

AI-powered content moderation system that monitors social platforms, broadcast media, and messaging apps for harmful election-related speech. Integrates with LiteLLM and social platform APIs.

## FastAPI Endpoints

- POST `/flags` - Submit content for classification
- GET `/flags/{id}` - Retrieve flag details
- POST `/flags/{id}/review` - Human review submission
- POST `/incidents` - Create safety incident report
- GET `/health` - Service health check

## Pydantic Models

- `ContentFlag` - Classification record (id, source_platform, content_hash, label, confidence)
- `SafetyIncident` - Incident report (id, flags: list[str], severity, description, evidence_package_id)
- `EvidencePackage` - Evidence collection (id, items: list, signed_hash, package_ts)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints
- `LITELLM_ADAPTER_URL` - LiteLLM API for model inference
- `SOCIAL_PLATFORMS_ADAPTER_URL` - Social platform API adapters
- `NCIC_ADAPTER_URL` - NCIC (National Cohesion Commission) reporting endpoint

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m speech_integrity.worker
```

## Content Labels

- `hate_speech` - Speech inciting violence against groups
- `disinformation` - False claims about election/voting
- `election_interference` - Calls to disrupt election
- `incitement` - Direct calls for violence or illegal acts

## Notes

- Kenyan-language bias is an open risk (see M17 notes); priority for bias testing
- Confidence > 0.85 flags for human review; > 0.95 auto-escalates
- NCIC incidents are submitted with full evidence package
- Social platform monitoring requires platform API credentials (Twitter, Facebook, WhatsApp)
