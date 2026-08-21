# ai-platform-svc

## Purpose

Hosts all AI/ML workloads: speech-integrity content moderation for Kenyan languages, anomaly and fraud detection across election telemetry, the AI model registry with governance controls, red-team adversarial testing, turnout forecasting, and a mandatory PII-stripping pipeline that wraps every model with audit logging. All model decisions are logged with AIDecisionLog records for post-election review.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M17 | speech-integrity | Content flagging, safety incidents, evidence packages; Kenyan-language bias mitigation |
| M26 | anomaly-fraud-detection | Detection alerts, telemetry packet analysis, cross-source comparison |
| M28 | ai-monitoring-stack | AI model registry, decision logs, model cards, PII stripping |
| M44 | turnout-forecasting | Forecast releases, hourly turnout reports, variance alerts |
| M61 | secrecy-assurance-testing | Red-team engagements, findings, IAB attestation |

## Key Aggregates and State Stores

- `ContentFlag` (PostgreSQL + object store): flagged content items with model scores and human review.
- `SafetyIncident` (PostgreSQL): escalated incidents with evidence packages.
- `DetectionAlert` (PostgreSQL): anomaly/fraud alerts with severity and cross-source correlation.
- `AIModel` (PostgreSQL): model registry entries with version, provenance, and card.
- `AIDecisionLog` (PostgreSQL, append-only): all model inference outputs.
- `ForecastRelease` (PostgreSQL): published turnout forecasts.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvi-svc | Identification event stream for anomaly detection |
| Upstream | rts-svc | Form transmission patterns for fraud signals |
| Upstream | ecc-svc | Alert acknowledgement and context |
| Upstream | field-mobile-svc | Field agent reports for cross-source comparison |
| Downstream | ecc-svc | Anomaly and fraud alerts |
| Downstream | governance-svc | Forensics referrals |
| External | NCIC | Speech-integrity escalations |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.ai_platform.alert.anomaly` | `eel.ai_platform` | Anomaly or fraud detection alert |
| `eel.ai_platform.content.flagged` | `eel.ai_platform` | Content flagged by speech-integrity model |
| `eel.ai_platform.forecast.published` | `eel.ai_platform` | Turnout forecast published |
| `eel.ai_platform.model.registered` | `eel.ai_platform` | New model version registered |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvi.identification.event` | bvi-svc |
| `eel.rts.form34.received` | rts-svc |
| `eel.ecc.alert.created` | ecc-svc |

## Primary APIs

- `packages/openapi-contracts/ai-anomaly/openapi.yaml` - anomaly detection query
- `packages/openapi-contracts/ai-forecast/openapi.yaml` - turnout forecast
- `packages/openapi-contracts/ai-registry/openapi.yaml` - model registry
- `packages/openapi-contracts/ai-redteam/openapi.yaml` - red-team engagement management

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Anomaly alert latency (event to alert) | less than 60 s |
| Speech-integrity classification P99 | less than 5 s |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h (model outputs are reproducible) |

## Degradation Modes

- If ML inference nodes are unavailable, alerts queue; ECC operates on direct telemetry.
- PII-stripping pipeline failure blocks model inference; audit log is never bypassed.
- Turnout forecasts fall back to historical baselines if live data is unavailable.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/incident-response-p0.md](../../runbooks/incident-response-p0.md)
