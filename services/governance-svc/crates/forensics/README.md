# eel-governance-forensics

M66 Election financial forensics: CBK/FRC API integration, anomaly detection ML models, suspicious transaction flagging.

## What it does

Detects election-related financial anomalies via machine learning models trained on historical election budgets. Models: transaction graph neural network (detects unusual payment flows), time-series LSTM (detects sudden expenditure spikes), isolation forest (detects outlier vendor patterns). Integrates with CBK (Central Bank of Kenya) and FRC (Financial Reporting Centre) to report suspected money-laundering and terrorism financing. Escalates high-confidence anomalies to financial crime task force for investigation.

Fits in governance-svc as the financial forensics layer. Runs on port 8095 (shared with OAG); consumes support.payroll.run.completed and support.budget.voucher.approved events; emits governance.forensics.anomaly.detected.

## Public API

`eel_governance_forensics::cycle::AnalysisCycle` aggregate: `fn ingest_transactions(&mut self, batch: TransactionBatch) -> Result<()>`, `fn run_detection(&mut self) -> Result<Vec<Anomaly>>`, `fn file_cbk_report(&mut self, anomaly: Anomaly) -> Result<()>`.

Endpoints:
- `GET /v1/governance/forensics/cycles/{id}` - fetch cycle status
- `POST /v1/governance/forensics/cycles/{id}/ingest` - batch upload transactions
- `POST /v1/governance/forensics/cycles/{id}/run-detection` - trigger anomaly run
- `GET /v1/governance/forensics/cycles/{id}/anomalies` - list detected (score > threshold)
- `POST /v1/governance/forensics/cycles/{id}/anomalies/{anomaly_id}/file-cbk-report` - escalate

Transaction format: JSON objects with vendor_id, amount, timestamp, transaction_type, approval_chain. Models run async; results cached in PostgreSQL for 30 days.

## Feature flags

`#[cfg(feature = "ml-inference")]` for live ONNX model inference (on-kit GPU); tests use CPU inference with smaller models.

`#[cfg(feature = "cbk-integration")]` for live CBK API calls; tests use httpserver stub.

## Test commands

```bash
cargo test -p eel-governance-forensics --all-features
cargo test -p eel-governance-forensics --lib ml_model_correctness
cargo test -p eel-governance-forensics --test integration_forensics -- --include-ignored
```

Inject synthetic anomalies; verify models detect them at configured thresholds.

## Bench commands

```bash
cargo bench -p eel-governance-forensics detect_100k_transactions
```

Running detection on 100k transactions should complete in < 5s (GPU acceleration).

## Dependencies of note

- `ort`: ONNX Runtime for model inference
- `uuid6`: cycle and anomaly IDs
- `tokio`: async CBK API calls, model inference
- `serde_json`: transaction and anomaly parsing
- `chrono`: transaction timestamps, cycle duration

## Related docs

- `docs/api-reference.md#forensics` (anomaly score interpretation)
- `docs/architecture.md#ml-models` (model architecture, training data)
- `py/forensics/models/` (model training code, hyperparameters)
- `migration_011_forensics.sql` (schema: analysis_cycles, transactions, anomalies, cbk_reports)
