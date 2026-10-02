# ai-platform-svc capabilities and algorithms

Comprehensive capability catalogue for the AI platform service. Every capability lists
the algorithm, source reference (paper, standard, or statutory authority), and the crate
or component that implements it.

## Contents

1. Modules and capabilities overview
2. End-to-end data flows
3. Seven-layer anomaly and fraud detection stack (M26)
4. CIB clustering for coordinated inauthentic behaviour (M17)
5. Deepfake detection facades (M17, M28)
6. Hourly turnout forecasting (M44)
7. PII stripping and regression corpus (M28)
8. Model cards, cosign-signed artefacts, and decision logs (M28)
9. Secrecy assurance red-team environment (M61)
10. LiteLLM and NLLB integration doubles
11. Operational excellence
12. Standards and interoperability
13. Validation harnesses
14. Data lifecycle and security
15. API surface
16. Summary

---

## 1. Modules and capabilities overview

| Module | Slug | Size | Primary crate | Notable risk |
|---|---|---|---|---|
| M17 | speech-integrity | XL | crates/ai-gateway + py/ | Kenyan-language bias; social API access |
| M26 | anomaly-fraud-detection | XL | crates/ai-gateway + py/ | alert fatigue; canary messaging |
| M28 | ai-monitoring-stack | XL | crates/ai-gateway + py/ | deepfake evasion; PII-strip regressions |
| M44 | turnout-forecasting | M | crates/ai-gateway + py/ | polling officer skips reports |
| M61 | secrecy-assurance-testing | M | crates/ai-gateway | staging isolation breach |

Tier: T2. Availability SLO: 99.9%. Anomaly alert delivery latency p95 < 30 s. Model inference latency p95 < 500 ms.

RTO: 60 min. RPO: 15 min.

---

## 2. End-to-end data flows

### 2.1 Anomaly detection pipeline (M26)

```
biometric-edge.identification.submitted (Kafka, at-least-once)
  -> crates/ai-gateway/src/anomaly_ingest.rs: parse TelemetryPacket
  -> py/anomaly_engine.py: 7-layer stack (see section 3)
  -> DetectionAlert persisted if any layer triggers
  -> eel.ai.alert_raised.v1 emitted to ecc-svc within 30 s (p95)
```

### 2.2 Speech integrity pipeline (M17)

```
Social platform feeds (Twitter, YouTube captions) + broadcast transcripts
  -> py/speech_monitor.py: language detection + CIB clustering + deepfake check
  -> ContentFlag persisted; EvidencePackage assembled
  -> eel.ai.speech_flag_raised.v1 emitted; M23 grievance channel notified if public harm
```

### 2.3 Turnout forecasting pipeline (M44)

```
Hourly: biometric-edge identification counts aggregated by station
  -> py/turnout_forecast.py: GBT model inference over cumulative_voters, time_of_day, weather, historical baseline
  -> ForecastRelease persisted; eel.ai.turnout_forecast_released.v1 emitted
  -> Public dashboard updated (portal-svc)
```

---

## 3. Seven-layer anomaly and fraud detection stack (M26)

### 3.1 Architecture

Python worker: `py/anomaly_engine.py`. Layers applied sequentially; any triggering layer creates a `DetectionAlert` with layer_id and evidence_json.

| Layer | Algorithm | Trigger condition |
|---|---|---|
| L1: Telemetry Isolation Forest | IsoForest (Liu, Ting, Zhou ICDM 2008) on kit telemetry features | anomaly_score > 0.85 |
| L2: Cross-source comparison | Signed chi-squared test between BVI counts and RTS Form34 tally | chi2 p-value < 0.01 |
| L3: Benford's law (first digit) | KS test on leading-digit distribution of station-level counts (Benford 1881) | KS D > 0.15, p < 0.05 |
| L4: Second-digit Benford | KS test on second-digit distribution (Nigrini 1996 "A taxpayer compliance application") | KS D > 0.12, p < 0.05 |
| L5: Cryptographic canary | Canary token inserted in training corpus; alert if model output reproduces canary | exact string match |
| L6: Convergence-divergence monitor | Jensen-Shannon divergence (Lin 1991 IEEE TIT) between current-window and baseline | JSD > 0.15 |
| L7: Cross-station spatial autocorrelation | Moran's I statistic (Moran 1950 Biometrika) on residuals after demographic adjustment | Moran's I p < 0.05 |

### 3.2 Alert severity

CRITICAL: L2 (cross-source) or L3/L4 (Benford) triggered simultaneously with L1. HIGH: any single layer triggers. MEDIUM: L5 canary or L6 divergence alone. Alert deduplication: same (station_id, layer_id, 10-min bucket) coalesced.

### 3.3 Kit-level L1 anomaly (Flutter plugin)

Component: `apps/` (Flutter plugin embedded in biometric-edge kit). Lightweight L1 Isolation Forest inference on-device (tflite model). Alerts uploaded to ai-gateway on connectivity; stored in `eel_kit_anomaly_events`.

| Algorithm | Reference |
|---|---|
| Isolation Forest | Liu, Ting, Zhou ICDM 2008 |
| Benford's law first-digit | Benford 1881 American Journal of Science |
| Second-digit Benford | Nigrini 1996 Journal of the American Taxation Association |
| Jensen-Shannon divergence | Lin 1991 IEEE Transactions on Information Theory |
| Moran's I spatial autocorrelation | Moran 1950 Biometrika |
| KS test | Kolmogorov 1933; Smirnov 1948 |

---

## 4. CIB clustering for coordinated inauthentic behaviour (M17)

### 4.1 Algorithm

Python worker: `py/speech_monitor.py`.

Coordinated Inauthentic Behaviour (CIB) detection clusters social media accounts by behavioural similarity to identify coordinated disinformation campaigns. Method:

1. Feature extraction per account: post timing distribution, content cosine similarity, retweet graph adjacency, hashtag co-occurrence vector.
2. Agglomerative hierarchical clustering (Ward linkage; Murtagh and Legendre 2014 Journal of Classification) with cosine distance on feature vectors.
3. CIB score per cluster: `mean(timing_sync_score, content_sim_score, graph_density_score)`. Clusters with CIB score > 0.75 flagged as coordinated.
4. Content from flagged clusters elevated for deepfake and narrative analysis.

| Algorithm | Reference |
|---|---|
| Ward agglomerative clustering | Ward 1963 JASA; Murtagh and Legendre 2014 Journal of Classification |
| Cosine similarity | Salton and McGill 1983 Introduction to Modern Information Retrieval |
| Timing synchronisation score | Ferrara et al. 2016 CACM (bot detection) |

---

## 5. Deepfake detection facades (M17, M28)

### 5.1 Detection pipeline

Python worker: `py/deepfake_detector.py`. Facade pattern: pluggable backend detectors.

| Backend | Modality | Reference |
|---|---|---|
| MesoNet | Face video | Afchar, Nozick, Yamagishi, Echizen WIFS 2018 |
| FaceForensics++ baseline | Face video | Rossler, Cozzolino, Verdoliva et al. ICCV 2019 |
| RawNet2 | Audio (voice deepfake) | Jung, Kim, Shim, Kim, Lee Interspeech 2020 |
| Frequency-domain texture analysis | Face image | Frank, Eisenhofer, Schiele, Fischer, Kolossa CVPR Workshops 2020 |

Detection facade: `DetectorBackend` protocol (Python ABC). Each backend implements `detect(media_bytes, media_type) -> DetectionResult { is_synthetic: bool, confidence: float, explanation: str }`. Results from all available backends merged; majority vote with confidence-weighted tiebreak.

### 5.2 Evidence packaging

When deepfake detected: `EvidencePackage` assembled with `{ media_hash, detection_results JSONB, detector_versions, detected_at }`. Package signed by ai-gateway HSM (key role `evidence_signer`). Forwarded to M23 grievance channel and transparency-svc for tlog anchoring.

---

## 6. Hourly turnout forecasting (M44)

### 6.1 Model

Python worker: `py/turnout_forecast.py`. Scikit-learn GradientBoostingRegressor (Friedman 2001 Annals of Statistics).

Features per station per hour:
- cumulative_voters_biometric: running BVI identification count.
- hour_of_day: integer 6-17.
- day_of_week: integer.
- historical_turnout_rate: station-level historical average from prior election cycles (where available).
- weather_category: {clear, cloudy, rain} from KMD API.
- queue_length: from queue-svc snapshot (optional; enriched when available).
- station_type: {urban, peri-urban, rural, diaspora}.

Target: `final_turnout_pct` (voters cast / registered voters).

Training: prior election datasets (M02 BVR historical register + BVI counts). Retrained each election cycle. Model stored in MinIO, Ed25519-signed, registered in model registry.

Inference: hourly batch job; outputs `ForecastRelease { election_id, forecast_timestamp, station_forecasts JSONB, national_point_estimate, national_ci_90 }`.

| Algorithm | Reference |
|---|---|
| Gradient Boosted Trees | Friedman 2001 Annals of Statistics |
| 90 % confidence interval via quantile regression | Koenker and Bassett 1978 Econometrica |
| KMD weather integration | Kenya Meteorological Department API |

---

## 7. PII stripping and regression corpus (M28)

### 7.1 PII stripping pipeline

Python worker: `py/pii_stripper.py`.

Applied before any data leaves the election domain into AI training corpora or model cards:

1. Named entity recognition: spaCy NER (Honnibal, Montani, Explosion AI 2017) detects PERSON, PHONE, ID_NUMBER, LOCATION (street level).
2. Rule-based patterns: regex for Kenyan national ID format (`[0-9]{7,8}`), phone (`07[0-9]{8}`), and MSISDN.
3. Replacement: detected spans replaced with `[REDACTED_<TYPE>]` placeholder.
4. Verification: stripped output re-run through NER; any residual PII detections trigger `PIIStrippingEvent { doc_id, residual_count, review_required }`.

### 7.2 Regression corpus

`py/pii_regression_corpus/`: 500 synthetic documents with known PII spans. Every PII stripping pipeline release must achieve: recall >= 0.99 on corpus (no PII leaked); precision >= 0.95 (acceptable redaction rate). Corpus tests run in CI: `cd py && uv run pytest tests/test_pii_regression.py`.

---

## 8. Model cards, cosign-signed artefacts, and decision logs (M28)

### 8.1 Model registry

Crate: `crates/ai-gateway/src/model_registry.rs`.

Every model registered in `eel_model_registry`:
- model_id (UUID7), name, version, algorithm, framework, training_date.
- artefact_url (MinIO path), sha256_digest, cosign_signature.
- model_card_url (PII-stripped), training_data_hash.
- deployment_status: {staging, canary, production, deprecated}.

Registration requires Ed25519 cosign signature from authorised MLOps principal (key role `model_publisher`). Unsigned models rejected.

### 8.2 Decision logs

Every inference that produces a `DetectionAlert` or `ForecastRelease` appended to `eel_ai_decision_logs`:
- decision_id (UUID7), model_id, input_hash (SHA-256 of input features), output_json, decision_timestamp, explanation_json.
- Anchored to transparency-svc tlog every 10 min via Merkle root (same pattern as inventory-svc).

### 8.3 cosign-signed model artefacts

Model artefact signing workflow:
```
1. Train model; export ONNX or pickle artefact
2. sha256sum artefact -> digest
3. cosign sign-blob --bundle artefact.bundle artefact
4. Upload artefact + bundle to MinIO
5. POST /v1/ai/models/register { sha256_digest, cosign_bundle_url }
6. ai-gateway verifies signature against Rekor before registering
```

| Algorithm | Reference |
|---|---|
| SHA-256 artefact digest | FIPS 180-4, NIST 2015 |
| Ed25519 cosign signing | Sigstore cosign; Bernstein et al. IACR ePrint 2011 |
| RFC 6962 inclusion proof | Rekor transparency log; Laurie, Langley, Kasper 2013 |

---

## 9. Secrecy assurance red-team environment (M61)

### 9.1 Staging isolation

Crate: `crates/ai-gateway/src/red_team.rs`.

Staging cluster isolated from production: separate Kubernetes namespace `ai-staging`, separate database, no network path to production services. Red-team engagements use synthetic data only; real voter data prohibited per ADR-0021.

`RedTeamEngagement` lifecycle:
```
Created -> Active (synthetic data loaded) -> InProgress (attack scenarios run)
  -> ReviewPending (findings assembled) -> IABAttested (IAB member signs findings)
  -> Closed
```

### 9.2 IAB attestation

Findings report signed by IAB member (SPIRE SVID + HSM, key role `iab_attester`). `IABAttestation { engagement_id, finding_summary_hash, attested_by, attestation_timestamp, hsm_signature }`. Stored in `eel_red_team_attestations`; forwarded to M15 IAB dashboard.

### 9.3 Attack scenario catalogue

Each `RedTeamEngagement` references one or more `AttackScenario` records drawn from the catalogue in `eel_attack_scenarios`:

| Scenario ID | Description | Target |
|---|---|---|
| AS-001 | Inference attack on aggregated count data | M26 anomaly outputs |
| AS-002 | Model inversion against turnout forecast | M44 GBT model |
| AS-003 | Adversarial count manipulation to evade Benford detection | M26 L3/L4 layers |
| AS-004 | CIB cluster poisoning via authentic-looking accounts | M17 CIB algorithm |
| AS-005 | Deepfake that passes all four detection backends | M17 deepfake facade |
| AS-006 | PII leakage through model card differencing | M28 PII stripper |

Findings for each scenario: severity (critical/high/medium/low), remediation_owner, remediation_deadline, patch_commit_sha. Unresolved critical findings block production model deployment (gate in model registry: `deployment_status` cannot advance from `staging` to `canary` while open critical finding exists for the model's scenario coverage).

---

## 10. LiteLLM and NLLB integration doubles

### 10.0 Bias and fairness assessment

Crate: `crates/ai-gateway/src/fairness.rs`.

All production anomaly detection models evaluated for demographic bias before each election cycle deployment. Assessment follows NIST AI RMF MAP-5 (bias source identification) and MEA-2 (bias metric computation):

| Demographic slice | Metric | Threshold |
|---|---|---|
| County (47) | False positive rate disparity across counties | max / min FPR ratio < 2.0 |
| Urban / rural / peri-urban / diaspora | Alert rate disparity | max / min ratio < 1.5 |
| Turnout decile | L3 Benford false positive rate per decile | no decile FPR > 2x population mean |

Bias assessment results stored in `eel_model_fairness_reports`: model_id, assessment_date, slice_type, slice_value, fpr, fnr, disparity_flag. Report signed by MLOps principal and forwarded to M15 IAB dashboard before deployment gate advances from `staging` to `canary`.

### 10.1 LiteLLM proxy double

Production path: LiteLLM proxy at `192.0.2.11:4000` (per infra SERVICE_CATALOGUE.md). In tests: `py/doubles/litellm_double.py` implements OpenAI-compatible `/v1/chat/completions` endpoint with deterministic fixture responses. Used by M17 speech integrity NLP tasks.

### 10.2 NLLB translation double

Production path: NLLB-200 (Meta AI 2022, 3.3B parameter multilingual model) served via `ml/vault` server. In tests: `py/doubles/nllb_double.py` returns pre-translated fixtures for the 11 required languages. Used by M68 referendum explainer TTS pipeline (invoked from portal-svc) and M17 multilingual content analysis.

---

## 11. Operational excellence

| Capability | Mechanism |
|---|---|
| Anomaly alert latency | Prometheus alert when p95 delivery > 30 s; consumer scaling via HPA |
| Model registry outage | Cached model version served; new registration disabled; alert raised |
| Inference timeout | Circuit breaker (tower middleware) with 5 s timeout; fallback to simplified heuristic rules |
| Speech monitor lag | Backpressure on social feed consumer; lag alert when > 10,000 messages behind |
| Turnout forecast stale | Alert if forecast_timestamp older than 90 min; fallback to last good forecast |
| PII stripping regression | CI gate: recall < 0.99 blocks model release |
| Staging isolation | Network policy enforced by Kubernetes NetworkPolicy; audit on cross-namespace traffic |
| HPA | CPU 70 %, memory 80 %; min 3, max 20 replicas |

---

## 12. Standards and interoperability

| Standard | Coverage | Crate / component |
|---|---|---|
| NIST AI RMF | AI Risk Management Framework; governs model card requirements, bias testing, deployment gates | crates/ai-gateway/src/model_registry.rs |
| ISO/IEC 23894:2023 | AI risk management; applied to anomaly detection and deepfake detection deployment decisions | crates/ai-gateway |
| PKCS#11 v3.0 | HSM interface for evidence signing and model publisher key | crates/ai-gateway/src/hsm.rs |
| CloudEvents 1.0 | All consumed and emitted event envelopes | crates/ai-gateway |
| OpenAPI 3.1 | API contract at packages/openapi-contracts/ai-platform/ | crates/ai-gateway |
| RFC 6962 | Decision log Merkle anchor to transparency-svc | crates/ai-gateway/src/decision_log.rs |
| OWASP ASVS L3 | Security verification for election-critical inference paths | crates/ai-gateway |

---

## 13. Validation harnesses

| Harness | Scope | Command |
|---|---|---|
| Isolation Forest unit tests | Anomaly score thresholds; feature scaling | `cd py && uv run pytest tests/test_anomaly.py` |
| Benford's law tests | KS test on synthetic fraudulent counts | `cd py && uv run pytest tests/test_benford.py` |
| PII regression corpus | Recall >= 0.99, precision >= 0.95 | `cd py && uv run pytest tests/test_pii_regression.py` |
| CIB clustering accuracy | Cluster purity on labelled test set | `cd py && uv run pytest tests/test_cib.py` |
| cosign signature verification | Model registration with and without valid bundle | `cargo test model_registry:: --lib` |
| Turnout forecast accuracy | MAPE vs holdout election data | `cd py && uv run pytest tests/test_forecast.py` |
| Decision log anchor | Merkle root publication + Rekor verification | `cargo nextest run --test decision_log_anchor` |
| k6 load test | 1000 VU anomaly ingest; p95 < 30 s | `cd loadtest && k6 run anomaly-ingest.js --vus 1000 --duration 300s` |

---

## 14. Data lifecycle and security

### 14.1 Encryption

Model artefacts: AES-256-GCM at rest in MinIO. Evidence packages: per-package DEK (AES-256-GCM), DEK wrapped under ai-platform KEK. Decision logs: no PII (input_hash is SHA-256 of features; no raw voter data).

### 14.2 Audit

`eel_ai_decision_logs` append-only. cosign signature on all model artefacts. IAB attestation HSM-signed. Red-team engagement events immutable in `eel_red_team_events`.

### 14.3 Retention

- `eel_anomaly_alerts`: 3 years post-election (election integrity archive).
- `eel_ai_decision_logs`: 7 years (statutory audit; Elections Act §39).
- `eel_model_registry`: permanent.
- `eel_turnout_forecasts`: 10 years (academic and audit reference).
- `eel_red_team_attestations`: permanent.
- `eel_pii_stripping_events`: 90 days (operational monitoring only).

### 14.4 Staging isolation

Production database and staging database are separate PostgreSQL instances with no shared roles. Staging data sourced from synthetic generator only. ADR-0021 prohibits real voter data in staging.

---

## 15. API surface

### 15.1 REST (`crates/ai-gateway`, axum 0.8, port 8080)

- `POST /v1/ai/anomaly/ingest`: submit telemetry batch for anomaly detection.
- `GET /v1/ai/alerts`: list anomaly alerts with temporal and severity filters.
- `GET /v1/ai/alerts/{id}`: single alert with evidence JSON.
- `GET /v1/ai/turnout/forecast/{election_id}`: latest hourly forecast with confidence intervals.
- `POST /v1/ai/models/register`: register cosign-signed model artefact.
- `GET /v1/ai/models`: list registered models with deployment status.
- `GET /v1/ai/models/{id}/card`: PII-stripped model card.
- `GET /v1/ai/red-team/engagements`: red-team engagement list (IAB-attested).
- `GET /healthz` + `/readyz` + `/openapi.json`.

All authenticated via SPIRE SVID. Idempotency-Key required on POST. Rate limit: 10,000 req/min per service identity for anomaly ingest; 100 req/min per user for model registration.

### 15.2 gRPC (`crates/ai-gateway/src/grpc`, port 8443)

`proto/ai-platform.proto`: IngestTelemetry, GetAnomalyAlerts, GetTurnoutForecast, RegisterModel RPCs.

### 15.3 Events (CloudEvents 1.0 via outbox)

Consumed:
- `eel.biometric_edge.identification_submitted.v1`
- `eel.rts.tally_updated.v1`
- `eel.ecc.alert_raised.v1` (feedback loop)

Emitted:
- `eel.ai.alert_raised.v1`
- `eel.ai.turnout_forecast_released.v1`
- `eel.ai.speech_flag_raised.v1`
- `eel.ai.model_registered.v1`

---

## 16. Summary

Total footprint: 1 Rust crate (ai-gateway) + 1 Flutter kit plugin (L1 anomaly) + Python workers (anomaly_engine, speech_monitor, turnout_forecast, pii_stripper, deepfake_detector, doubles) + PostgreSQL (migrations placeholder, schema in data_dictionary.yaml). Covers M17 (speech integrity with CIB clustering and deepfake detection facades), M26 (7-layer anomaly stack combining IsoForest, Benford analysis, cross-source comparison, and spatial autocorrelation), M28 (model registry with cosign-signed artefacts, PII stripping with regression corpus, and decision logs anchored to tlog), M44 (hourly GBT turnout forecast), and M61 (staging-isolated secrecy assurance red-team with IAB attestation). Key production inputs remaining: social platform API access for M17 (Twitter Academic Research API + YouTube Data API); NLLB-200 model deployment on ml/vault server; NIST AI RMF formal assessment before production deployment.
