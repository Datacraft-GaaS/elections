# Financial Forensics ML

Financial forensics ML service for campaign finance analysis. Runs statistical analysis cycles (Benford's law, network graph analysis, time-series anomaly detection) on candidate and party financial data. Generates referral packets for CBK/FRC (Financial Reporting Centre).

## Overview

Module: M66 (Financial Forensics)

Statistical fraud detection service for campaign finance. Identifies suspicious financial patterns and prepares evidence packages for regulatory submission.

## FastAPI Endpoints

- POST `/analysis/start` - Initiate forensic analysis cycle
- GET `/analysis/{id}` - Retrieve analysis status and results
- POST `/analysis/{id}/referral` - Generate CBK/FRC referral packet
- GET `/health` - Service health check

## Pydantic Models

- `AnalysisCycle` - Analysis job (id, election_id, method: str, status, created_at)
- `EstimateRecord` - Estimated hidden funds (cycle_id, entity_id, estimated_amount: float, confidence)
- `ReferralPacket` - Evidence package (cycle_id, entity_id, evidence: list, referred_to: str, created_at)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints
- `CBK_FRC_ADAPTER_URL` - CBK/FRC referral submission endpoint
- `LITELLM_ADAPTER_URL` - LiteLLM API for LLM-assisted analysis

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m forensics_ml.worker
```

## Analysis Methods

1. **Benford's Law**: First-digit frequency analysis (detects round numbers, inflated amounts)
2. **Network Graph**: Transaction flow analysis (detects money laundering chains)
3. **Time-Series Anomaly**: Deviation from expected spending patterns

## Notes

- Confidence scores indicate statistical certainty (0-1.0)
- Estimates are probabilistic; used for investigative leads only
- Referrals are submitted with full evidence (not prosecution)
- Analysis requires 500+ transactions for reliable detection
- Post-election analysis only (during campaign finance review window)
