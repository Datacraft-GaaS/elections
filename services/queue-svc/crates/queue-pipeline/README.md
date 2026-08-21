# queue-pipeline

Real-time queue snapshot aggregation and disenfranchisement detection. Fuses kit signals, field mobile updates, and CV head-count into unified QueueSnapshot. Emits DisenfranchisementFlag and S14ARecommendation events.

Sits at the core of queue-svc as the aggregation engine: consumes multiple data sources, computes station snapshots, detects anomalies, recommends election extensions.

## What it does

Maintains 30s rolling window snapshots per station by fusing kit signals (from biometric edge), field mobile reports, and optional CV head-count. Computes aggregated queue length, average wait time, closure risk. Detects queue anomalies (spikes, stalls, CV mismatches). Flags disenfranchisement risk and recommends S.14A election extension when queue closure threatens voter access.

## Public API

Exposed modules via lib.rs:
- snapshot: QueueSnapshot, aggregate_signals(stream) -> Snapshot
- fusion: fuse_kit_field_cv(kit_signal, field_update, cv_estimate) -> FusedValue
- anomaly: detect_anomaly(snapshot) -> Option(Anomaly)
- s14a: compute_s14a_risk(snapshot) -> S14ARecommendation

## Feature flags

None currently.

## Testing

Unit tests (aggregation, anomaly detection):
```bash
cargo test --lib
```

Fusion logic correctness (multi-source merging):
```bash
cargo test fusion::
```

S.14A recommendation thresholds:
```bash
cargo test s14a::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Snapshot aggregation (10k signals per 30s window):
```bash
cargo bench aggregate_10k_signals
```

## Dependencies of note

Workspace:
- eel-events-rs: Anomaly and recommendation events
- eel-observability-rs: Aggregation metrics

External:
- tokio: Async runtime
- chrono: Timestamp windowing
- uuid: Anomaly tracking IDs

## Related docs

See services/queue-svc/docs/queue-fusion-strategy.md for multi-source merge logic.
See services/queue-svc/docs/s14a-risk-model.md for closure time and disenfranchisement thresholds.
See docs/runbooks/queue-anomaly-investigation.md for troubleshooting.
