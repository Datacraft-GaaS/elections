# ai-platform-svc

## 1. Overview

ai-platform-svc hosts the platform's AI/ML workloads: a Rust inference gateway (model
registry, anomaly alert API with SSE streaming, decision log) with mandatory PII stripping,
a telemetry-analytics detector engine, Python ML services (speech integrity, anomaly
detection and training, turnout forecasting, cross-platform CIB detection, model registry,
secrecy red-team, adversarial training, monitoring), an Elixir speech-analyst LiveView
console, and a Flutter L1 telemetry collector for kit devices.

- **Runtime**: Rust (axum: ai-gateway on `0.0.0.0:8082`, telemetry-analytics admin router),
  Python (FastAPI services per py/* package), Elixir LiveView (ex/speech_analyst),
  Flutter (apps/kit-l1).
- **Ports**: ai-gateway hard-codes 8082 (crates/ai-gateway/src/main.rs:7). Python services
  expose FastAPI apps (uvicorn defaults); telemetry-analytics router has no binary in this
  crate (library). Other ports (not verified).
- **Entrypoints**: crates/ai-gateway/src/main.rs; each py/*/src/*/service.py FastAPI app;
  ex/speech_analyst/lib/speech_analyst_live/application.ex; apps/kit-l1 Flutter app.
- No TRACEABILITY.yaml or service-level tests directory exists
  (services/ai-platform-svc/tests is empty).

## 2. Ownership & Boundaries

Ground truth: docs/architecture/services/ai-platform.md.

| M-ID | Module | Implementation |
|---|---|---|
| M17 | speech-integrity | py/speech-integrity + ex/speech_analyst console |
| M26 | anomaly-fraud-detection | crates/telemetry-analytics, py/anomaly(+training), apps/kit-l1 |
| M28 | ai-monitoring-stack | py/model-registry, py/ai-monitoring, crates/ai-gateway |
| M44 | turnout-forecasting | py/turnout-forecast |
| M61 | secrecy-assurance-testing | py/secrecy-redteam (+py/adversarial-training) |

- **Data ownership**: ai_anomaly_alerts/adjudications/ingested_events, turnout forecasts +
  accuracy reports, retrain jobs, redteam campaigns/findings, registry models/deployments/
  audit log, request fingerprints (migrations/V1__ai_platform_contracts.sql).
- **Upstream**: bvi-svc (identification events), rts-svc (form transmission patterns),
  ecc-svc (alert context), field-mobile-svc (field reports). External: NCIC escalations.
- **Downstream**: ecc-svc (anomaly/fraud alerts), governance-svc (forensics referrals).
- Events documented (`eel.ai_platform.alert.anomaly`, `.content.flagged`,
  `.forecast.published`, `.model.registered`); no Kafka wiring found in code (stub).

## 3. Repository Layout

```
services/ai-platform-svc/
  crates/
    ai-gateway/src/        # lib.rs (AppState+app), routes/{mod,registry,anomaly,
                           #   decision_log}.rs, pii_strip.rs, models.rs
    ai-platform-api/store/ # store helpers (no binary)
    ai-ensemble-voting/    # ensemble.rs, policy.rs, registry.rs (voting ensemble library)
    telemetry-analytics/   # api.rs (admin rules/detectors/replay), detectors/, pipeline.rs,
                           #   backtest.rs, dispatch.rs, metrics.rs
  py/
    speech-integrity/      # service.py, views.py, detectors.py
    anomaly/, anomaly-training/, adversarial-training/
    cross-platform-cib/    # corr.py, graph.py, models.py
    model-registry/        # service.py (FastAPI registry), views.py
    secrecy-redteam/       # engagements/findings API
    turnout-forecast/      # forecaster.py, service.py
    ai-monitoring/         # service.py, views.py
  ex/speech_analyst/       # Phoenix LiveView dashboard/flags/adjudication
  apps/kit-l1/             # Flutter L1 telemetry collector (M26)
  migrations/V1__ai_platform_contracts.sql
  config/telemetry-rules.yaml   # default detector rule set
```

## 4. Capabilities & Functions

### ai-gateway HTTP API (port 8082)

App nests `/v1/ai` over three sub-routers (crates/ai-gateway/src/lib.rs:18–23,
src/routes/mod.rs:10–15):

Registry (/v1/ai/registry/*, src/routes/registry.rs:18–20):
| Method | Path | Handler (line) | Purpose |
|---|---|---|---|
| GET/POST | /models | `list_models` :23 / `register_model` :43 | Model catalogue / registration |
| GET | /models/{id} | `get_model` :30 | Model detail |
| PATCH | /models/{id}/activate | `activate_model` :69 | Activation gate |

Anomaly (/v1/ai/anomaly/*, src/routes/anomaly.rs:22–25):
GET/POST /alerts (`list_alerts` :28 / `ingest_alert` :67), GET /alerts/{id} (:35),
PATCH resolve (:53), GET /stream — SSE alert stream (`stream_alerts` :76).

Decision log (/v1/ai/decision-log/*, src/routes/decision_log.rs:19):
GET/POST /entries (`list_entries` :22 / `create_entry` :40) — append-only AI decision log
per docs/architecture/services/ai-platform.md (AIDecisionLog).

PII pipeline: `strip_pii()` replaces emails → `[EMAIL_REDACTED]` and digit runs >= 7 →
`[ID_REDACTED]`, returning replacement count (crates/ai-gateway/src/pii_strip.rs:23);
unit tests at :57–72.

### telemetry-analytics admin API

Router (crates/telemetry-analytics/src/api.rs:53–61): GET/POST /v1/analytics/rules
(`list_rules` :66-ish / `upsert_rule`; handlers follow route defs), GET/DELETE
/v1/analytics/rules/{id}, GET /v1/analytics/detectors/{id}/status,
POST /v1/analytics/replay (`ReplayRequest` :24). Detector implementations under
detectors/ (DropAtScaleDetector, GeoFenceBreachDetector, SessionHijackDetector per
config/telemetry-rules.yaml).

### Python services (FastAPI)

Model registry exemplar endpoints (py/model-registry/src/model_registry/service.py):
health :42, list/get models :47/:52, model card :60, register :71, activate :105,
decision-log list/create :128/:134. Secrecy red-team: health :34,
POST /v1/redteam/engagements :40 (+findings/IAB attestation flows in same module).
Speech integrity, anomaly, turnout forecast, CIB, and monitoring packages follow the same
service/views layout (service.py holds the FastAPI app and domain functions, e.g.
py/anomaly/src/anomaly/service.py `detect(points)` :44, `list_alerts` :57).

### Elixir console

Live routes (ex/speech_analyst/lib/speech_analyst_live/router.ex:20–27): `/dashboard`
(DashboardLive), `/flags` (FlagsLive), `/adjudication` (AdjudicationLive),
`GET /` health controller.

### kit-l1 collector

Flutter background service collecting GPS/accelerometer/battery/network/scan-timing
telemetry and forwarding to the anomaly backend; performs no inference locally
(apps/kit-l1/README.md).

## 5. Data Model

Single migration V1__ai_platform_contracts.sql:

| Table | Purpose |
|---|---|
| ai_anomaly_alerts (:9) | Detection alerts with severity/correlation fields |
| ai_anomaly_adjudications (:28) | Human adjudication of alerts |
| ai_ingested_events (:42) | Raw ingested telemetry/events |
| ai_turnout_forecasts (:55) | Forecast releases |
| ai_forecast_accuracy_reports (:74) | Post-hoc accuracy tracking |
| ai_retrain_jobs (:89) | Retraining queue |
| ai_redteam_campaigns (:107) | Red-team engagements |
| ai_redteam_findings (:127) | Findings with IAB attestation |
| ai_registry_models (:148) | Registry entries (version/provenance/card) |
| ai_registry_deployments (:166) | Deployment records |
| ai_registry_audit_log (:181) | Append-only registry audit trail |
| request_fingerprints (:198) | Request deduplication/idempotency fingerprints |

Python services persist in-process (log-style Vec/list structures, e.g.
py/anomaly/src/anomaly/service.py `_log_alert` :28) rather than PostgreSQL.

Events documented: emitted/consumed as listed in §2; not wired.

## 6. Operation

- Build/run: `cargo run -p ai-gateway` (8082); run each FastAPI service via its package
  (uvicorn entrypoints in py/*/src/*/service.py); `mix phx.server` for speech_analyst;
  flutter build/run for kit-l1.
- Env vars: none read by ai-gateway main (hard-coded addr); RUST_LOG honoured via
  tracing_subscriber::fmt::init() (main.rs:4–6). Detector configuration from
  config/telemetry-rules.yaml (rule ids drop_at_scale, geo_fence_breach, session_hijack
  with severity thresholds and params).
- Test: cargo unit tests embedded (pii_strip tests, ensemble/policy modules); pytest suites
  per py package (tests/ci/test_*.py); flutter test for kit-l1.
- Deploy: services/ai-platform-svc/deploy/service.yaml.
- Degradation modes documented in docs/architecture/services/ai-platform.md; batch
  analytics tolerate backlog by design.

## 7. Safeguards & Security

- **Auth posture**: ai-gateway attaches only TraceLayer (lib.rs:19–22) — no auth
  middleware; registry activation, decision-log writes, and alert resolution are open to
  any network caller. Python FastAPI services define no auth dependencies in inspected
  files (not verified exhaustively). This is the dominant security gap.
- **PII handling**: mandatory PII stripping is implemented and unit-tested
  (pii_strip.rs:23,:57–72), matching the "PII-stripping pipeline wraps every model"
  requirement in docs/architecture/services/ai-platform.md. However, gateway routes do not
  visibly call strip_pii before forwarding (not verified that it is enforced end-to-end).
- **Audit logging**: append-only ai_registry_audit_log table and decision-log endpoint;
  `_log_register/_log_activate` hooks in model-registry service.py:34,:38.
- **Idempotency**: request_fingerprints table exists; consumer code absent.
- **Rate limiting**: none found in code.
- **Validation**: serde/pydantic request models; DB CHECK constraints in V1 migration.
- **Tests**: pytest suites across all nine py packages (tests/ci/test_*.py), including
  benford/isolation-forest convergence checks (py/anomaly-training/tests/) and ensemble
  voting tests; Rust inline tests; no auth/security suites found.
- **Known deferrals**: no authentication anywhere; PII strip not provably wired into the
  request path; Kafka event emission absent; ai-platform-api crate reduced to a store
  helper with no binary; empty top-level tests/ directory; TRACEABILITY.yaml missing while
  sibling services have one.

## 8. Observability

- Tracing: tower-http TraceLayer on the gateway (lib.rs:21); tracing_subscriber fmt init.
- Metrics: telemetry-analytics exposes metrics.rs helpers; no Prometheus endpoint found.
- Health: gateway has no dedicated /health route (not found); Python services expose
  per-package /health (e.g. model-registry service.py:42, redteam :34); Elixir console has
  `GET /` HealthController (router.ex:27).
