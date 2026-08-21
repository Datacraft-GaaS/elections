# eel-abis-threshold-tuning

Per-modality, per-cohort decision threshold tuning for the eel ABIS.

## Algorithm

Threshold selection follows the ROC operating-point method:

1. Score all labelled pairs (genuine + impostor) for a cohort.
2. Build the full ROC curve by sweeping all unique score thresholds.
3. Select the highest threshold whose false match rate (FMR) is at or below
   the configured target (default 0.001%, i.e. 1 in 100,000).
4. Bootstrap 95% CIs (n=1000 resamples) provide stability bounds.

Cohort key: `(modality, age_band, sex, county_code)`. Kenya has 47 counties
plus a national (code=0) fallback. Age bands: child/young_adult/adult/senior.

## Integration Points

- **eel_threshold_profiles** table: profiles persisted via `ThresholdRepo`.
- **activate_profile**: atomically flips `active=TRUE` for one profile and
  deactivates all siblings in the same cohort (transactional).
- **Simulate**: stateless dry-run via `ThresholdTuner::tune` without DB writes.
- Consumer: `abis-matcher` reads the active profile for each (modality, cohort)
  at match time to apply the correct acceptance threshold.

## SLA Impact

Threshold updates take effect on next match request after activation. Expect
FMR changes up to 0.05% relative if CI bounds are wide (few pairs). Recommend
at least 1000 genuine and 1000 impostor pairs per cohort for stable estimates.

## Usage

```rust
use eel_abis_threshold_tuning::tuner::ThresholdTuner;
use eel_abis_threshold_tuning::domain::LabelledPair;

let tuner = ThresholdTuner::default(); // n_bootstrap=1000
let result = tuner.tune(&pairs, 0.00001)?;
println!("threshold={:.6} FMR={:.6}", result.threshold, result.achieved_fmr);
```

## Modules

| Module | Purpose |
|--------|---------|
| `tuner` | `compute_roc`, `select_threshold`, `ThresholdTuner::tune` |
| `domain` | `LabelledPair`, `ThresholdProfile`, `BootstrapCi`, `SimulateResult` |
| `repo` | `ThresholdRepo`: sqlx CRUD against `eel_threshold_profiles` |
| `activate` | `activate_profile`: atomic cohort activation |
| `api` | `simulate`: stateless dry-run entry point |
