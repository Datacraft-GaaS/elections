# Anomaly and fraud detection (M26)

Module M26 (`anomaly-fraud-detection`) owned by `ai-platform-svc`. Operates a 7-layer detection stack: Layer 1 telemetry Isolation Forest, Layer 2 cross-station ensemble, Layer 3 Beber-Scacco last-digit uniformity (Beber and Scacco, American Journal of Political Science 56(1), 2012), Layer 4 distributional forensics (Rozenas 2017; Klimek et al., PNAS 109(41), 2012), Layer 5 cryptographic canary, Layer 6 PVT triangulation, and Layer 7 tip convergence. An ECC console routes officer adjudication. A public detection report is published after each election.

## Contents

1. What anomaly and fraud detection does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What anomaly and fraud detection does (vs related modules)

- **M21 RLA audit tool** = physical ballot statistical verification; M26 is computational signal analysis on digital telemetry and declared results.
- **M26** = 7-layer algorithmic detection applied to live telemetry during and after polling; signals are fused and routed to ECC adjudication; no automated referral occurs without officer review.
- **M23 Grievance channels** = public tip intake; Layer 7 fuses M23 tips with algorithmic signals for convergence scoring.

M26 produces `AnomalySignal` records. No M26 output directly modifies a certified result or triggers a prosecution. The ECC console is the mandatory human gate.

## 2. Data plane

Five aggregates under `services/ai-platform-svc/crates/anomaly-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **AnomalySignal** | Detection output: signal_id (UUID7), layer (L1-L7), source, score, threshold, status (OPEN/ADJUDICATED/DISMISSED), model_version. | B |
| **LayerResult** | Per-layer computation: layer, contest_id or station_id, metric_value, p_value (where applicable), interpretation, computed_at. | B |
| **CanaryEvent** | Cryptographic canary verification: canary_id, expected_hash, observed_hash, match (bool), ts_hlc. | A |
| **AdjudicationRecord** | Officer decision: signal_id, officer_id, decision, evidence_ref, ts_hlc, sig. | A |
| **DetectionReport** | Post-election public report: report_id, election_cycle, layer_summaries (JSON), signal_count, public_sig, published_at. | D |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Station telemetry (biometric events, voter throughput) | Real-time | Kafka | SVID mTLS | Buffer; Layer 1 runs on available batch; gap logged |
| M09 declared result totals | Per tabulation event | Kafka | SVID mTLS | Layers 3-4 run only after M09 events; gap logged |
| Cryptographic canaries (M05 P05 tlog) | Every Merkle root | Internal | SVID mTLS | Layer 5 failure is a P0 alert |
| PVT (parallel vote tabulation) external sources | Post-poll | REST | JWT (IAB) | Layer 6 runs only after PVT data received; gap OK |
| M23 grievance tips | Real-time | Kafka | Internal | Layer 7 runs on any FINANCE or CONDUCT tip |

## 4. Pipeline

```
Layer 1 (Telemetry Isolation Forest -- Liu, Ting, Zhou, ICDM 2008):
  station telemetry windowed per 30 min
  -> IsolationForest(n_estimators=200, contamination=0.01)
     trained on historical baseline; scored per station
  -> score > threshold: AnomalySignal(L1)

Layer 2 (Cross-station ensemble):
  L1 scores aggregated across constituency
  -> IsolationForest ensemble with cross-station features
  -> score > threshold: AnomalySignal(L2)

Layer 3 (Beber-Scacco last-digit test -- Beber and Scacco, AJPS 56(1), 2012):
  declared totals per candidate per station
  -> extract last digits; expected uniform distribution U(0,9)
  -> chi-squared goodness-of-fit; p < 0.01: AnomalySignal(L3)

Layer 4 (Distributional forensics):
  method 1: Rozenas (2017, J. Politics) -- regression discontinuity at 50%
  method 2: Klimek et al. (PNAS 109(41), 2012) -- 2D fingerprint scatter
  -> outlier station clusters: AnomalySignal(L4)
  NOTE: Benford 2BL applied only for corroboration; Deckert, Myagkov,
        Ordeshook (Electoral Studies 2011) caution against primary reliance
        on Benford for small constituency counts; result noted but not used
        as standalone trigger.

Layer 5 (Cryptographic canary):
  each P05 Merkle root contains a canary leaf (known hash)
  -> CanaryEvent: expected_hash vs observed_hash
  -> mismatch: P0 alert; AnomalySignal(L5, severity=CRITICAL)

Layer 6 (PVT cross-source triangulation):
  IAB submits PVT data (parallel volunteer counts)
  -> per-station z-score between PVT and M09 declared total
  -> |z| > 3.0: AnomalySignal(L6)

Layer 7 (Tip convergence):
  M23 FINANCE and CONDUCT tips aggregated per constituency
  -> constituency with >= 3 L1/L2/L3 signals AND >= 2 M23 tips
     within the same polling day window:
     AnomalySignal(L7, convergence_score = f(signals, tips))

ECC console adjudication:
  officer reviews AnomalySignal; selects: DISMISS / ESCALATE / REFER
  REFER: eel.anomaly.referral_triggered.v1 (exactly-once)
  DetectionReport assembled post-election; public_sig (Chair HSM)
```

## 5. API surface

Contract: `packages/openapi-contracts/ai-platform/anomaly.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /anomaly/v1/signals` | ECC officer (JWT + SpiceDB) | Active signal feed with score and layer |
| `PATCH /anomaly/v1/signals/{id}` | ECC officer | Adjudicate signal |
| `GET /anomaly/v1/layers/{layer}/results` | IAB / ECC officer | Layer computation results for a contest |
| `GET /anomaly/v1/canaries` | ECC / ops (JWT) | Canary verification feed |
| `POST /anomaly/v1/pvt` | IAB officer (JWT) | Submit PVT data for Layer 6 |
| `GET /anomaly/v1/report/{cycle}` | Public (CDN-cached) | Post-election detection report |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| ECC console (Phoenix LiveView) | `services/ai-platform-svc/ex/anomaly_web/` | Live signal feed; layer breakdown; adjudication controls; canary status |
| IAB PVT intake form (Astro) | `services/ai-platform-svc/astro/pvt-portal/` | IAB uploads PVT data for Layer 6 |
| Public detection report (Astro) | `services/ai-platform-svc/astro/detection-public/` | Post-election public report; aggregated, no station-level PII |

WCAG 2.2 AA on all surfaces. Languages: EN + SW.

## 7. Integrity + anti-tamper

- **Model artefact signing** -- all detection models (IsolationForest weights, threshold configs) are cosign-signed; the ECC console verifies the cosign signature before loading any model; unsigned models are rejected.
- **Decision log anchoring** -- every `LayerResult` and `AdjudicationRecord` is hashed and anchored in the P05 transparency log within 60 s; third parties can verify no signal was silently suppressed.
- **PII stripping circuit-breaker** -- no voter or officer PII is passed to any ML model or LLM; the circuit-breaker (`packages/eel-pii-strip-rs`) is applied before any data reaches the model layer; a PII detection event halts the pipeline and alerts the DPO.
- **Canary P0 alert** -- a Layer 5 canary mismatch triggers an immediate P0 page to the IEBC SOC; the ECC console highlights the affected Merkle root; no adjudication step is required before the alert fires.
- **Isolation Forest baseline** -- the contamination parameter and training window are stored in the model registry; any change to these parameters requires a new cosign-signed model release; ad-hoc parameter changes are rejected.

## 8. Kenya statute alignment

- **Elections Act s.82 (electoral offence reporting)** -- Layer 7 convergence signals routed to ECC adjudication provide the evidentiary basis for s.82 referrals; no referral is automated.
- **DPA 2019 s.40 (automated processing)** -- M26 does not make any decision that affects a voter's rights; it produces advisory signals for human adjudication; the DPA automated-processing review covers the ECC adjudication workflow, not the detection models themselves.
- **POCAMLA 2009** -- Layer 3 and Layer 4 signals related to result manipulation are cited in any M23 FINANCE referral package assembled under EOA ss.6-12.
- **DPA 2019 s.26 (minimisation)** -- station-level aggregates used in layers 1-4; no individual voter records or biometric data pass through the detection pipeline.

## 9. Honest caveats + external prerequisites

- Beber-Scacco last-digit test is not reliable for small sample sizes (fewer than ~200 tallies per candidate per contest level); Layer 3 is suppressed for contests below this threshold and the suppression is noted in the public report.
- Klimek et al. (2012) 2D fingerprint method was developed on large national elections; applicability to constituency-level Kenyan counts has not been empirically validated; results are treated as corroborative, not conclusive.
- IsolationForest contamination parameter is calibrated on historical Kenyan election data; if no historical baseline is available for a new constituency, Layer 1 runs in observation-only mode with no threshold alerts.
- PVT data availability for Layer 6 depends on IAB fielding a parallel tabulation operation; this is an operational prerequisite outside M26's scope.
- The public detection report timeline (published within 30 days of results certification) requires that all AdjudicationRecords are complete; delays in ECC adjudication delay the report.

## 10. Test coverage + operational runbooks

Coverage under `services/ai-platform-svc/crates/anomaly-api/tests/` and `crates/anomaly-models/tests/`: unit tests for each layer's computation (L1-L7); statistical unit tests for Beber-Scacco chi-squared (known-uniform and known-non-uniform inputs); regression corpus for IsolationForest threshold; property-based tests asserting L5 canary mismatch always triggers CRITICAL signal; contract tests (Pact) for M09 result events, M23 tip events, P05 canary feed; integration tests with model artefact registry, cosign verification, PII strip circuit-breaker, ECC console submission; e2e scenario covering live telemetry through L1-L7, ECC adjudication, and report assembly; load tests (k6) for 50k station telemetry events per minute.

Runbooks under `docs/runbooks/`:

- `anomaly-detection-degradation.md` (create): model load failure; P05 canary mismatch P0; PVT data late; ECC console unavailable.
- `l5-canary-incident.md` (create): immediate response for cryptographic canary mismatch.

Dashboards: `platform/observability/ai-platform-svc/grafana-anomaly.json` (signal rate by layer, canary status, adjudication queue depth, model version in use). SLO: canary verification within 5 min of Merkle root; L1 computation within 5 min of telemetry window close; ECC adjudication queue <100 open signals.

Related: `docs/plans/modules/anomaly-fraud-detection.md`; `services/ai-platform-svc/docs/CAPABILITIES.md`; edge-case matrix EC-AFD-* in `docs/plans/08-edge-case-matrix.md`.
