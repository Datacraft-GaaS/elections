# eel-abis-drift-monitor

Feature-space distribution drift detection for the eel ABIS model pipeline.

## Statistics

Four complementary statistics computed per feature per rolling window:

| Metric | Interpretation |
|--------|----------------|
| PSI | Population Stability Index: < 0.1 stable, 0.1-0.25 warning, > 0.25 critical |
| KS | Kolmogorov-Smirnov: max CDF difference, 0=identical |
| JS | Jensen-Shannon divergence: symmetric, bounded [0, ln 2] |
| Wasserstein | Earth mover's distance (1D CDF integral) |

## Windows

Three rolling windows run concurrently against the same baseline snapshot:
24h (operational), 7d (weekly trend), 30d (monthly trend).

## Alert Flow

1. `DriftDetector::detect` computes all four statistics.
2. PSI threshold determines severity (Info / Warning / Critical).
3. `maybe_alert` emits a `DriftAlert` row for Warning+ and logs via tracing.
4. After 7+ days of persistent drift, `suggest_adaptive_threshold` proposes
   a new PSI warning threshold = 1.5x the median observed PSI.

## Integration Points

- **eel_drift_alerts** table: alert records with resolution tracking.
- Consumer: model monitoring dashboard and on-call alerting pipeline.
- Baseline snapshots: captured during model training/validation phase.

## Modules

| Module | Purpose |
|--------|---------|
| `detector` | PSI, KS, JS, Wasserstein functions and `DriftDetector::detect` |
| `domain` | `FeatureDistribution`, `DriftReport`, `DriftAlert`, `DriftSeverity` |
| `alert` | `maybe_alert`: conditional alert creation |
| `adaptive` | `suggest_adaptive_threshold`: median-based threshold proposal |
| `window` | `window_start`, `aggregate_distributions` |

## Usage

```rust
use eel_abis_drift_monitor::detector::DriftDetector;
use eel_abis_drift_monitor::domain::DriftWindow;

let detector = DriftDetector::default();
let report = detector.detect(&baseline, &current, DriftWindow::D7)?;
```
