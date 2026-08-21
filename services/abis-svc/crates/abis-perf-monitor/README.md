# eel-abis-perf-monitor

Continuous biometric performance monitoring for the eel ABIS.

## Metrics

- **FMR** (False Match Rate): fraction of impostor pairs scoring above threshold.
- **FNMR** (False Non-Match Rate): fraction of genuine pairs scoring below threshold.
- **Rank-N Accuracy** (FRVT-style): fraction of genuine probes where the mate
  appears in the top-N gallery results.

## Schedule

PMR runs every 6 hours against a labelled holdout set held out from training.
Holdout pairs are reviewed quarterly and refreshed.

## Rollup Reports

Daily, weekly, and monthly rollups aggregate PMR runs into trend reports stored
in the reports catalogue. Rollups compare current metrics against configured
performance targets and alert if any target is breached.

## Integration Points

- **eel_performance_reports** table: report records.
- Labelled holdout: maintained in a separate S3 prefix.
- Consumer: operations dashboard, compliance reporting.

## Performance Targets (defaults)

| Metric | Target |
|--------|--------|
| FMR | <= 0.001% |
| FNMR | <= 2% |
| Rank-1 | >= 99% |
