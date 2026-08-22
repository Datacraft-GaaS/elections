# Turnout forecasting analytics (M44)

Module M44 (`turnout-forecasting-analytics`) owned by `ai-platform-svc`. Produces hourly turnout forecasts during election day using gradient boosted tree models trained on historical turnout data plus real-time station queue signals; publishes `ForecastRelease` artefacts with uncertainty bands; triggers variance alerts when observed vs. predicted diverges beyond threshold; and requires double-verification by the Kenya Meteorological Department (KMD) weather integration before each release.

## Contents

1. What M44 does (vs M16 ECC and M10 RTS real-time tally)
2. Data plane (schema)
3. Model and data inputs
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M44 does (vs M16 ECC and M10 RTS real-time tally)

- **M10 RTS** = real-time tally of votes cast; it counts results, not turnout rates.
- **M16 ECC** = election command centre situational awareness; it aggregates incident reports and logistics status.
- **M44** = forward-looking turnout prediction, updated hourly on election day. It informs resource reallocation decisions (printer re-supply, officer redistribution, kit battery logistics) and feeds M16's ECC dashboard. It does not influence vote counting or result declaration.

M44 outputs are explicitly advisory. No dispatch or tally action is taken automatically based on a forecast; all actions require a human ECC decision.

## 2. Data plane

Three aggregates (per `services/ai-platform-svc/crates/ai-gateway/src/models/forecast.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ForecastRelease** | Published forecast snapshot: `release_id` (UUID7), `election_cycle`, `release_hour` (0-12, election day hours), `contest_scope` (`NATIONAL / COUNTY / CONSTITUENCY`), `model_version`, `kmd_double_verified` (boolean), `released_at`, `overall_predicted_turnout_pct`, `ci_lower_pct`, `ci_upper_pct`, `release_sha256`, `hsm_signature`. | B |
| **HourlyTurnoutReport** | Per-constituency observed input: `report_id` (UUID7), `release_id`, `constituency_code`, `queue_depth` (from M32 queue transparency), `bvi_scans_so_far` (from M06), `weather_risk_score` (from KMD), `predicted_final_turnout_pct`, `prediction_std`. | B |
| **VarianceAlert** | Alert when observed diverges from predicted: `alert_id` (UUID7), `release_id`, `constituency_code`, `predicted_pct`, `observed_pct_at_alert_time`, `divergence_pct`, `alert_threshold_pct` (configurable, default 10), `alerted_at`, `resolved_at`, `resolution_note_hash`. | B |

## 3. Model and data inputs

Gradient Boosted Trees (Chen and Guestrin, "XGBoost: A Scalable Tree Boosting System", KDD 2016) trained on:

| Feature group | Source | Cadence |
|---|---|---|
| Historical turnout by constituency + hour of day | IEBC historical DB | Pre-election batch |
| Queue depth at each station | M32 queue transparency API | Hourly pull |
| BVI scan count (proxy for actual votes cast) | M06 BVI backend API | Hourly pull |
| Weather risk score (rain, heat index) | KMD API (`integrations/kmd-weather/`) | Hourly pull |
| Registered voter count per constituency | M35 `RegisterSnapshot` | Pre-election batch |
| Population density + urbanisation index | KNBS census data | Pre-election batch |
| Distance-to-station index | M18 boundary + ward centroid data | Pre-election batch |

The model produces a predicted final turnout percentage per constituency with a 90% confidence interval. Uncertainty quantification follows Abdar et al. ("A review of uncertainty quantification in deep learning", Information Fusion 2021) adapted for gradient boosted ensembles via bootstrap aggregation (20 sub-models, `n_estimators=200` each).

## 4. Pipeline

```
pre-election: model training pipeline (Python, services/ai-platform-svc/py/)
      -> feature engineering from historical + registration data
      -> XGBoost training (Chen & Guestrin, KDD 2016)
      -> model artifact stored in MLflow registry (services/ai-platform-svc/docs/mlflow.md)
      -> model_version recorded; model frozen before T-7 (election freeze)

election day: hourly forecast job (cron via packages/eel-scheduler-rs)
      -> pull HourlyTurnoutReport inputs:
            -> queue_depth from M32 API
            -> bvi_scans from M06 API
            -> weather_risk_score from KMD API
      -> KMD double-verification:
            -> KMD API queried twice (5-min gap); if responses diverge >15%: hold release
            -> kmd_double_verified flag set true if both pulls agree
      -> run model inference -> predicted_final_turnout_pct + CI per constituency
      -> aggregate to national level -> overall_predicted_turnout_pct + CI
      -> create ForecastRelease + HourlyTurnoutReport rows
      -> release_sha256 computed; HSM signature applied
      -> emit eel.ai.forecast_released.v1
      -> M16 ECC dashboard updated via SSE
      -> variance check: compare predicted_pct vs observed (bvi_scans / registered)
            -> if divergence > alert_threshold_pct: VarianceAlert created
            -> alert sent to ECC duty officer via Slack + SMS
      -> release stored for public dashboard (aggregate national + county only)
```

## 5. API surface

Contract: `packages/openapi-contracts/ai-forecast/` (path determined at OpenAPI contract generation). All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/forecast/releases` | M16 ECC (SVID) or commission admin | List ForecastRelease records for current cycle |
| `GET /v1/forecast/releases/{release_id}` | M16 ECC (SVID) | ForecastRelease detail + constituency breakdown |
| `GET /v1/forecast/releases/{release_id}/reports` | Commission admin | HourlyTurnoutReport rows for release |
| `GET /v1/forecast/alerts` | ECC duty officer (JWT) | List VarianceAlerts; filter by resolved |
| `PATCH /v1/forecast/alerts/{alert_id}/resolve` | ECC duty officer (JWT) | Resolve alert + note |
| `GET /v1/forecast/public` | Public (60/hr) | National + county aggregate only; no constituency detail |
| `POST /v1/forecast/releases/{release_id}/retract` | Commission admin + HSM | Retract a release (e.g., model error detected) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| ECC forecast dashboard (integrated into M16 ECC) | `services/ecc-svc/ex/ecc_web` | Hourly forecast charts; variance alert feed |
| AI platform admin (Django) | `services/ai-platform-svc/django/` (if present) | Model version management; release audit |
| Public forecast page (Astro, part of M11 portal) | `services/portal-svc/astro/gazette` or public portal | National + county aggregate forecast; updated hourly |

Languages: EN + SW. ECC dashboard is EN-only (internal). Public forecast page is EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Model freeze at T-7** - the model version is locked in the MLflow registry at T-7 (7 days before election). Any post-freeze model change requires Commissioner + HSM authorisation and is logged in `eel_governance_audit_shadow`.
- **KMD double-verification** - two independent KMD API calls 5 minutes apart; divergence >15% holds the release and alerts ops. This guards against a transient KMD data error inflating or deflating the weather risk score.
- **HSM-signed releases** - `ForecastRelease.hsm_signature` is verifiable against the Commissioner public key. A tampered release (e.g., predicted_turnout_pct edited in the DB) is detectable by signature verification.
- **Release SHA-256** - computed over the canonical JSON serialisation of all `HourlyTurnoutReport` rows for the release. Any post-sign mutation is detectable.
- **Retraction audit** - retracted releases are not deleted; they are flagged `RETRACTED` with a Commissioner HSM countersignature. The original release data is preserved for post-election audit.
- **Variance alert non-suppression** - the alert system cannot be disabled via API; only a code change can raise the threshold. The threshold is a configurable environment variable with a minimum enforced value of 5%.

## 8. Kenya statute alignment

- **Elections Act §44** - Commission has a mandate to inform the public; the public forecast page satisfies the informational duty without disclosing voter-level data.
- **Kenya Data Protection Act §26 (minimisation)** - no voter PII in forecast artefacts; only aggregate constituency-level counts.
- **Kenya Data Protection Act §41 (automated processing)** - forecast outputs are advisory; no automated decisions are made. The spec explicitly labels M44 outputs as advisory and requires human ECC decision for any resource reallocation.
- **Statistics Act Cap 416** - KNBS census data used as model input is public-domain aggregate data; no individual-level census data is used.

## 9. Honest caveats + external prerequisites

- Model accuracy is bounded by the quality of historical turnout data. If past elections had significant data quality issues (unreported stations, stuffed registers), the model inherits those biases. A model bias audit is recommended before each election cycle.
- KMD API integration (`integrations/kmd-weather/`) requires a KMD API key provisioned under IEBC. The adapter is ready; the live key is an external dependency.
- Queue depth from M32 is available only where queue-monitoring kits are deployed. Constituencies without M32 coverage have their `queue_depth` imputed using the historical median; imputed inputs are flagged in `HourlyTurnoutReport.queue_depth_imputed`.
- The 90% CI is model-derived and does not account for systemic events (mass violence, voter suppression, infrastructure failure). ECC officers should treat the CI as a statistical range, not a safety guarantee.
- MLflow model registry requires an MLflow server instance. The deployment config at `services/ai-platform-svc/deploy/` assumes a self-hosted MLflow on the ml/vault server (62.169.25.77).

## 10. Test coverage + operational runbooks

Tests under `services/ai-platform-svc/tests/`: model inference with synthetic election-day data, KMD double-verification logic (mock API), variance alert threshold property-based tests via `pytest + hypothesis`, HSM signature verification.

Runbooks under `docs/runbooks/`:

- `ai-platform-svc.md` - forecast job deploy, model rollback, KMD adapter failure.
- `forecast-retraction.md` - retraction procedure on model error detection.
- `kmd-api-down.md` - fallback behaviour (last good weather score re-used; release marked `KMD_STALE`).

Related documents: spec at `docs/plans/modules/`; capability catalogue at `services/ai-platform-svc/docs/`; M16 ECC at `docs/features/`; M32 queue transparency at `docs/features/`.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*

**Performance targets**: All read endpoints respond within 200ms at p95 under 100 concurrent users. Write endpoints (including HSM-signing paths) respond within 1,000ms at p95. Bulk operations (roster upload, CSV import) are async with a job status poll endpoint.

**SLA targets**: All API reads respond within 200ms p95. Event processing latency from trigger to state update is under 5 seconds under normal load. HSM-signing operations complete within 3 seconds.

**Availability target**: Routine 99.5% outside election windows; election-day window 99.9%. Managed via `packages/eel-resilience-rs` circuit breakers on all external integrations.
