# Anomaly Detection

Anomaly and fraud detection ML service. Processes kit telemetry data, performs cross-source validation (BVI vs RTS vs portal), and raises `DetectionAlert` events to ECC (Election Command Centre) when anomalies are detected.

## Overview

Module: M26 (Anomaly Fraud Detection)

ML-powered fraud prevention system that monitors election kit behavior and result consistency, flagging suspicious patterns for ECC investigation.

## FastAPI Endpoints

- POST `/telemetry/ingest` - Ingest kit telemetry packets
- GET `/alerts/{id}` - Retrieve alert details
- GET `/health` - Service health check

## Pydantic Models

- `DetectionAlert` - Alert record (id, severity, alert_type, evidence: list, created_at)
- `TelemetryPacket` - Kit metrics (kit_id, ts, metrics: dict, source)
- `CrossSourceComparison` - Comparison result (sources: list, discrepancy_score: float, findings: list)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints
- `LITELLM_ADAPTER_URL` - LiteLLM API for anomaly scoring models
- `ECC_API_URL` - Election Command Centre API for alert submission
- `ALERT_THRESHOLD` - Anomaly score threshold (0.0-1.0, default 0.85)
- `BVI_API_URL` - Biometric Verification Instance API
- `RTS_API_URL` - Results Transmission System API

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m anomaly.worker
```

## Notes

- Cross-source validation compares biometric counts, result tallies, and voter turnout
- High-severity alerts (score > 0.95) are immediately escalated to ECC
- Telemetry is sampled (not all kits, selected set) to avoid performance impact
- ML models are retrained weekly with new labeled anomalies
