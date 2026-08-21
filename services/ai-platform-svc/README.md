# AI Platform Service

AI-driven election integrity monitoring: speech integrity analysis, real-time anomaly detection, turnout forecasting, and secrecy assurance testing.

## Purpose and modules

Operates T2 election AI platform comprising five modules: M17 speech-integrity monitors social media and broadcast content for disinformation and speech attacks (with Kenyan-language bias mitigations); M26 anomaly-fraud-detection fuses telemetry from voting endpoints for real-time outlier alerting to ECC; M28 ai-monitoring-stack manages model lifecycle (registry, PII stripping, drift detection, signature validation); M44 turnout-forecasting-analytics produces hourly election forecasts from biometric-edge submissions; M61 secrecy-assurance-testing operates red-team engagement environment for in-house and IAB-attested security validation in staging isolation.

Tier: T2.

Availability SLO: 99.9%. Anomaly alert delivery latency p95 < 30s. Model inference latency p95 < 500ms.

RTO: 60 min. RPO: 15 min.

## Architecture overview

Python ONNX inference pipeline backed by Rust API gateway. Core aggregates: AnomalyAlert, ModelCard, TurnoutForecast, SpeechFlag. Ingestion layer consumes biometric-edge.identification.submitted events for baseline training; real-time data flows to anomaly detection engine (isolation forests, statistical bounds, rule-based heuristics). Speech monitor pipeline processes text feeds from social APIs (Twitter, YouTube captions) via transformer models. Model registry enforces Ed25519 signatures on all production artifacts. Deployment layer uses Rust gateway for rate-limiting per consumer, request validation, and audit logging. Turnout forecast refreshes hourly from cumulative biometric data. Secrecy assurance environment (staging-isolated cluster) accepts red-team engagement configurations and runs controlled chaos scenarios.

Upstream dependencies: biometric-edge-svc (identification.submitted events), rts-svc (tally.updated for forecast feedback), ecc-svc (alert.raised for feedback loop).

Downstream consumers: ECC alert dashboard, public transparency portal (forecasts), secrecy assurance validation framework.

## Directory layout

```
services/ai-platform-svc/
  apps/                   - Flutter kit plugin (L1 anomaly detection)
  crates/
    ai-gateway/           - Rust API gateway (inference routing, model validation)
  deploy/                 - Kustomize base + overlays
  docs/                   - Service documentation and runbooks
  ex/                     - Elixir dashboards (if present)
  migrations/             - PostgreSQL schema versioning
  py/                     - Python ONNX inference, model registry, speech pipeline
  src/                    - Main Rust binary
  tests/                  - Integration tests
  Cargo.toml, Cargo.lock
  data_dictionary.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/ai-platform/):
- POST /v1/ai/anomaly/ingest: Submit telemetry for anomaly detection
- GET /v1/ai/alerts: List anomaly alerts with temporal filtering
- GET /v1/ai/alerts/{id}: Retrieve single alert with evidence
- GET /v1/ai/turnout/forecast/{election_id}: Hourly turnout forecast
- POST /v1/ai/models/register: Register signed model artifact
- GET /v1/ai/models: List registered models with metadata
- GET /v1/ai/models/{id}/card: Retrieve model card (PII-stripped)

gRPC (proto/ai-platform.proto):
- IngestTelemetry(telemetry_batch) -> IngestResult
- GetAnomalyAlerts(time_window) -> AlertStream
- GetTurnoutForecast(election_id, timestamp) -> ForecastSnapshot
- RegisterModel(model_bytes, signature) -> ModelId

## Key events emitted and consumed

Emitted:
- ai.alert.raised: Anomaly detected and alerted to ECC
- ai.model.registered: Model artifact registered and indexed
- ai.turnout.forecast.released: Hourly forecast computed and published
- ai.speech.flag.raised: Speech integrity violation detected

Consumed:
- biometric-edge.identification.submitted: Baseline training data for anomaly detection
- rts.tally.updated: Updated tally for forecast refinement
- ecc.alert.raised: Feedback loop to improve detection rules

## Data model

Consult migrations/ for versioned PostgreSQL schema. data_dictionary.yaml defines:
- anomaly_alerts: alert_id, timestamp, alert_type, evidence_json, severity, ecc_ack_timestamp
- model_registry: model_id, name, version, signature, registered_at, pii_stripped_card
- turnout_forecasts: forecast_id, election_id, timestamp, forecast_json, confidence_interval
- speech_flags: flag_id, content_hash, platform, detected_lang, violation_type, timestamp

Model artifacts stored in MinIO with Ed25519 signature validation.

## Building and running

Build the Rust gateway:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/ai-platform-svc
cargo build --release
```

Build Python environment and run ONNX inference server:
```bash
cd py
uv sync
uv run python -m ai_platform.server --config dev.toml
```

Run full stack locally:
```bash
cargo run --bin ai-platform-svc -- --config dev.toml &
cd py && uv run python -m ai_platform.server &
```

Build Flutter kit plugin:
```bash
cd apps && flutter pub get && flutter build aar --release
```

## Testing

Unit and property tests (Rust):
```bash
cargo test --lib --all
cargo test --lib -- --test-threads=1  # For integration-style unit tests
```

Integration tests (requires PostgreSQL, testcontainers):
```bash
cargo nextest run --package ai-platform-svc
```

Contract tests (OpenAPI compliance):
```bash
cargo test --test '*_contract'
```

Python inference tests:
```bash
cd py && uv run pytest tests/ -v
```

Load testing (k6):
```bash
cd loadtest && k6 run anomaly-ingest.js --vus 1000 --duration 300s
```

Test tags: unit, integration, contract, load, chaos.

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service deployment, PDB, HPA
- deploy/overlays/primary: Primary site
- deploy/overlays/dr: Disaster recovery site

HPA targets CPU 70%, memory 80%; min replicas 3, max 20.

PDB: minAvailable 2 (always 3+ replicas running).

ApplicationSet at deploy/applicationset.yaml triggers Kustomize overlay build per environment.

## Observability

Dashboards (Grafana):
- Anomaly Detection: ingestion rate, alert rate, latency p50/p95/p99
- Model Registry: model registration rate, signature validation failures
- Turnout Forecast: forecast refresh interval, confidence distribution
- Speech Monitor: content processing rate, violation detection rate

Alerts (Prometheus):
- Anomaly alert delivery latency > 30s: warning
- Model inference latency p95 > 500ms: warning
- Model signature validation failure rate > 1%: critical
- PostgreSQL replication lag > 5 min: warning

Key metrics:
- ai_anomaly_alert_delivery_latency_seconds
- ai_model_inference_latency_seconds
- ai_model_registration_rate
- ai_turnout_forecast_refresh_interval_seconds
- ai_speech_flag_detection_rate

## Runbooks

See docs/runbooks/ai-platform-degradation.md for:
- High anomaly alert latency recovery
- Model registry outage (fallback to cached models)
- Speech monitor pipeline stuck (manual retry)
- Turnout forecast computation lag
- PostgreSQL failover

## Degradation contract

Normal: Full inference, alerting, forecast, speech monitoring.

Degraded: Pre-computed forecasts served from cache; real-time anomaly detection from simplified heuristics only; speech monitor pauses pending recovery.

Read-only: Serve last alerts, forecasts, and model cards; no new inference; model registration disabled.

## Chaos scenarios

- model-registry-unavailable: Kill model registry service; verify cached model usage
- inference-timeout: Saturate inference queue; verify p95 latency threshold
- speech-monitor-lag: Delay social feed ingestion; verify backpressure
- turnout-forecast-stale: Stop biometric data flow; verify forecast staleness alert
- anomaly-alert-batch-loss: Simulate Kafka partition; verify alert recovery

## Statutes

Elections Act §39(1C) transmission audit trail.
Elections (Technology) Regulations 2017 election systems integrity.
Data Protection Act anomaly detection privacy safeguards.

## Follow-ups

1. Implement federated learning for multi-site model refinement (phase 2).
2. Add real-time speech monitor dashboard with human-in-loop review.
3. Extend turnout forecasting to 15-min cadence during peak voting.
4. Integrate ICAO 9303 biometric template binding for model-based verification.
5. Implement model drift detection and automated retraining pipeline.
