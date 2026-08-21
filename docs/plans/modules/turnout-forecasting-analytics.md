# M44 turnout-forecasting-analytics

| field | value |
|---|---|
| M-number | M44 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.5 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5%; 99.5% on public dashboard during polling hours |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M44-turnout-forecasting-analytics.md |

## Purpose

Pre-election turnout forecasting from CVR demographics, historical turnout, and macro signals (weather, transport, registration density). Election-day turnout tracking with variance alerts. Post-election analytical dataset for M41 post-election review and academic/journalist access. Turnout variance alerts feed M41 integrity review and M16 operational response per Strategy §XIX.1.

Strategy reference: §XIX.1 (Anomaly Detection), §XIX.3 (Post-Election Review). DPA 2019.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Public Turnout Dashboard (WCAG 2.2 AA) | Astro | services/ai-platform-svc/astro/turnout-dashboard/ |
| IEBC Internal Analytics Console | Phoenix LiveView | services/ai-platform-svc/ex/analytics_web/ |
| API | Rust (axum) | services/ai-platform-svc/crates/ai-platform-api/ |
| Model Training Pipeline | Python (async) | services/ai-platform-svc/py/turnout_forecasting/ |

Languages: EN + SW; public dashboard WCAG 2.2 AA; forecast methodology document in EN + plain-language SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ForecastRelease | id, election_id, published_at, model_version, training_dataset_hash (SHA-256 of CVR extract), constituency_forecasts (FK list) | C | T2 |
| ConstituencyForecast | id, forecast_release_id, constituency_code, registered_voters, forecast_turnout_pct, interval_80_low, interval_80_high, interval_95_low, interval_95_high | C | T2 |
| HourlyTurnoutReport | id, polling_station_code, constituency_code, report_hour (7-17 EAT), voters_turned_out, submitted_at, presiding_officer_id | C | T2 |
| TurnoutVarianceAlert | id, constituency_code, alert_hour, observed_pct, forecast_pct, deviation_pct, sigma_deviation, alerted_at, acknowledged_by, acknowledged_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/ai-platform/turnout.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/forecasts/latest | 1 | public | n/a | latest pre-election forecast all constituencies |
| GET /v1/forecasts/{release_id} | 1 | public | n/a | specific forecast release |
| GET /v1/turnout/live | 5 (SSE) | public | n/a | election-day live turnout per constituency (SSE stream); updated every 30 min |
| GET /v1/turnout/alerts | 2 | JWT+SpiceDB (OPERATIONS_ANALYST/COMMAND_CENTRE) | n/a | active variance alerts (IEBC internal) |
| POST /v1/turnout/alerts/{id}/acknowledge | 2 | JWT+SpiceDB (OPERATIONS_ANALYST) | yes | acknowledge alert |
| GET /v1/datasets/post-election | 4 | JWT+SpiceDB (PUBLIC/JOURNALIST/ACADEMIC) | n/a | post-election constituency-level dataset; academic sub-county requires DPO approval |
| POST /v1/datasets/academic-request | 2 | JWT+SpiceDB (ACADEMIC) | yes | request academic sub-county dataset; triggers DPO review |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.analytics.forecast_published.v1 | emits | eel.analytics.turnout.v1 | operational | yes | at-least-once idempotent |
| eel.analytics.turnout_variance_alert.v1 | emits | eel.analytics.turnout.v1 | operational | yes | exactly-once |
| eel.analytics.post_election_dataset_published.v1 | emits | eel.analytics.turnout.v1 | operational | yes | at-least-once idempotent |
| eel.analytics.hourly_turnout_ingested.v1 | consumes | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M02 BVR Back-End | internal | integrations/bvr/ | yes | CVR demographic aggregate extract; no individual records retained |
| M08 RTS Device | internal | integrations/rts-device/ | yes | hourly Presiding Officer turnout reports; missing reports trigger RO supervisor alert |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | turnout anomaly feed as operational layer; circuit breaker |
| M41 Post-Election Review | internal | integrations/review/ | yes | anomaly-correlation JSON report within T+30; queued if M41 not ready |
| M50 By-Election Module | internal | integrations/by-election/ | yes | by-election historical turnout as additional baseline; optional |
| KMD Weather API | external | integrations/kmd/ | yes (stub) | county weather forecast for pre-election model; static seasonal-average fallback pre-loaded |
| P05 Merkle Log | P05 | integrations/merkle/ | yes | dataset publication hash anchoring |

## ML and AI

Model class: B1 (central analytics model, ADR-0018). Gradient-boosted ensemble (e.g. XGBoost) retrained each election cycle. Inputs: CVR demographic aggregates, historical turnout time series (2013/2017/2022 general + by-elections), KMD weather forecast, KURA road-accessibility index, voter density per station. Outputs: constituency-level turnout point estimate + 80%/95% prediction intervals.

Hard constraints:
- No individual voter data enters the model at any stage; only aggregate demographic summaries.
- Model version, training dataset hash, and feature list stored per release; pipeline reproducible from archived inputs.
- Forecast methodology document published alongside each release in plain-language EN + SW.
- Model not used for any decision in the vote-counting or tally pipeline (ADR-0018 hard red line).

## Dependencies

- Modules: M02, M08, M16, M41, M50
- Platform: P05
- Packages: eel-outbox-rs, eel-resilience-rs
- Python packages: see `services/ai-platform-svc/py/turnout_forecasting/pyproject.toml`

## Feature checklist

1. FR-M44-001: ingest CVR demographic aggregates per constituency (age-band, gender, disability flag, registration date) from M02; no individual records retained
2. FR-M44-002: ingest historical turnout per constituency (2013, 2017, 2022 general + by-elections from M50) as baseline time series
3. FR-M44-003: ingest macro signals: KMD weather forecast, KURA road-accessibility index, voter density per station
4. FR-M44-004: train and serve constituency-level turnout forecast model; outputs point estimate + 80%/95% prediction intervals; retrained each cycle
5. FR-M44-005: forecast published at T-30, T-14, T-7; each publication versioned and archived; methodology document published alongside
6. FR-M44-006: forecast methodology in plain-language EN + SW
7. FR-M44-007: Presiding Officers submit hourly voter count per station via M08 RTS device; data flows to aggregation pipeline
8. FR-M44-008: constituency-level turnout computed every hour; compared against forecast trajectory
9. FR-M44-009: variance alert: observed turnout deviates > 2 SD or > 15 percentage points from forecast at any constituency at any hour; alert to M16 and IEBC Operations Analyst within 5 minutes
10. FR-M44-010: public turnout dashboard updated every 30 minutes during polling hours (07:00-17:00 EAT); constituency-level only; no station-level during polling hours
11. FR-M44-011: final per-station turnout published after polls close and Form 34A data received; reconciled against CVR registered voter count
12. FR-M44-012: post-election dataset (per-constituency turnout, demographic breakdown, weather index, historical delta, anomaly flags) available T+7 via registered data-access API
13. FR-M44-013: journalist API: read-only; aggregated per constituency; no individual voter data; rate-limited; no registration required
14. FR-M44-014: academic dataset API: sub-county breakdown on request with IEBC DPO approval; DPA 2019 s.30(1)(e) research lawful basis; data-sharing agreement required
15. FR-M44-015: anomaly-correlation report cross-referencing turnout anomaly flags with M41 integrity events; delivered to M41 Review Officer as structured JSON within T+30

## Acceptance criteria

1. AC-M44-01: T-30 forecast published; methodology document live; point estimates and intervals correct; back-test MAE <= 5 percentage points on 2022 data
2. AC-M44-02: election-day variance alert delivered to M16 within 5 minutes of trigger; integration test
3. AC-M44-03: public dashboard updated every 30 minutes during polling hours; p99 <= 2 s under 50,000 concurrent users; load test
4. AC-M44-04: post-election dataset available at T+7; Merkle hash published; researcher can verify dataset against hash
5. AC-M44-05: academic sub-county request triggers DPO review; access granted only after approval; security test

## Applicable edge cases

**connectivity**
- EC-CON-03: presiding officers skip hourly reports; M08 device prompts at each hour mark; RO supervisor receives non-submission alert; constituency turnout shows "data pending" on public dashboard

**operations**
- EC-OPS-05: KMD weather API unavailable at T-7; static seasonal-average fallback dataset pre-loaded; forecast proceeds; methodology document notes fallback source used

**scale**
- EC-SCL-02: public dashboard overwhelmed at poll-close rush; CDN caches 30-minute snapshots; dataset download rate-limited; IEBC variant behind authentication scales separately

**legal**
- EC-LEG-02: academic sub-county dataset requires DPO approval and data-sharing agreement; no sub-county data served without confirmed approval; DPA 2019 s.30(1)(e)

**ML**
- EC-ML-01: forecast model underperforms in low-registration constituencies; prediction intervals widened; low-sample constituencies flagged on dashboard; M41 notified if variance alerts disproportionately cluster in flagged constituencies

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | forecast aggregation pipeline, variance alert threshold computation, hourly ingestion idempotency, dataset hash | nextest + pytest | M44-UNIT- |
| back-test | run model against 2022 election data; MAE <= 5 percentage points vs published turnout | pytest (offline) | M44-BACKTEST- |
| contract | M08 hourly report ingest, M16 alert push, M41 correlation report, P05 hash anchor | Pact | M44-PACT- |
| integration | T-30 forecast publish; election-day pipeline; variance alert; T+7 dataset publication | integration_test | M44-INT- |
| e2e | simulate polling hours; PO submits hourly; variance alert; public dashboard update | nextest e2e | M44-E2E- |
| security | academic sub-county access without DPO approval (deny); dataset hash tamper detection | ZAP + red team | M44-SEC- |
| a11y | WCAG 2.2 AA on public dashboard; EN + SW | axe-core | M44-A11Y- |
| load | 50,000 concurrent public dashboard reads during polling hours | k6 | M44-LOAD- |

Tags: `#[trace("FR-M44-009","AC-M44-02","EC-ML-01")]`

## Runbooks and dashboards

- `docs/runbooks/ai-platform-svc-degradation.md` - extend with turnout forecast pipeline failures and KMD fallback procedure
- `docs/runbooks/turnout-variance-alert-response.md` - new runbook; variance alert escalation; M16 coordination; IEBC Commissioner notification SLA
- Grafana dashboard: `eel-iems-44` (alert ruleset: `iems-44-alerts`)
- SLO alerts: variance alert delivery <= 5 min; public dashboard update <= 30 min; post-election dataset published by T+7

## Open questions

- CVR demographic aggregate extract: M02 team must confirm the aggregate schema and API authentication before T-270; extract must be privacy-preserving (no individual records).
- M08 RTS device hourly turnout field: M08 firmware must include the structured hourly count field; confirm timeline with M08 team (target T-90).
- KMD API access agreement: Kenya Meteorological Department API requires MoU; target T-180; static seasonal-average fallback covers any delay.
- Academic dataset DPO batch-review capacity: standard data-sharing agreement template and pre-published batch DPO review slots to be arranged by T-7 post-election.
