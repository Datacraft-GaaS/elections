# eel-abis-model-canary

Blue-green model canary rollout with automatic SLA-based rollback for the eel ABIS.

## Algorithm

Progressive rollout schedule: 0% -> 10% -> 25% -> 50% -> 100%, with a
15-minute hold at each step before auto-advancing. Traffic routing is
deterministic per subject_id (FNV-1a hash mod 100), so each subject always
hits the same model throughout a canary run, eliminating flip-flop effects.

## Integration Points

- **eel_model_canaries** table: canary records with state and metrics.
- **dispatcher**: called at match time to select baseline vs. canary model.
- **monitor**: called by the metrics collector to evaluate SLA conditions.
- **auto-rollback**: if FMR/FNMR/latency_p99/error_rate breaches the SLA
  configured at canary start, rollback triggers automatically.
- **promote**: moves canary to 100% and marks the old model deprecated.

## SLA Configuration

Configure per-canary at start time. Suggested baselines:

| Metric | Default |
|--------|---------|
| FMR limit | 0.01 (1%) |
| FNMR limit | 0.05 (5%) |
| Latency p99 | 500ms |
| Error rate | 1% |

## Modules

| Module | Purpose |
|--------|---------|
| `canary` | `start_canary`, `next_step`, `try_advance` |
| `dispatcher` | `route_to_canary`, `select_model` |
| `monitor` | `check_sla` returning `Option<RollbackReason>` |
| `promote` | `promote` |
| `rollback` | `rollback` (idempotent) |

## Usage

```rust
use eel_abis_model_canary::canary::start_canary;
use eel_abis_model_canary::dispatcher::select_model;

let canary = start_canary(old_id, new_id, 10, 0.01, 0.05, 500.0, 0.01)?;
let model_id = select_model(subject_id, baseline_id, Some(&canary));
```
