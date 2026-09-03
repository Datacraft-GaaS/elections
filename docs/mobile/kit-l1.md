# kit-l1 (Anomaly Plugin) — Integration Plan

Master plan: `docs/mobile/plan.md`. Module: kit-l1 on-device IsoForest
plugin (M26/M28 support). Est: 2 ED.

## Current state

`services/ai-platform-svc/apps/kit-l1/` — 5 lib files. `EelKitL1AnomalyDetector`
(pure-Dart isolation scoring) + models. Not consumed by bvi-kit/rts-kit; no
telemetry bridge to `telemetry-ingest` anomaly events.

## Work items

| # | Item | Details |
|---|---|---|
| L1 | Publish as kit-sdk plugin surface | move detector API under `eel_kit_sdk/anomaly` re-export (keep package as plugin host) |
| L2 | Wire into bvi-kit + rts-kit | subscribe to kit telemetry points (votes/spoilt ratios), fit per-station baseline (rolling 100 events), score each new point, score z > 3 → `anomaly_flag` outbox event → BFF `tamper-events`/anomaly ingest |
| L3 | Fail-safe | detector never blocks capture/submit (advisory only); un-fitted state = score null |
| L4 | Calibration | CI fixture corpus (synthetic normal + injected anomalies) → report detection AUC ≥ 0.75, false-positive ≤ 2 % at chosen threshold |

## Files

```
packages/kit-sdk-flutter/lib/src/anomaly_plugin.dart   (re-export + integration hooks)
services/bvi-kit/lib/src/telemetry/anomaly_watcher.dart        NEW
services/rts-svc/apps/rts-kit/lib/src/rts_workflow/anomaly_watch.dart  NEW
services/ai-platform-svc/apps/kit-l1/test/detection_auc_test.dart
```

## Testing

- Unit: featurization, rolling fit, z-score thresholds; AUC fixture test.
- Integration: bvi-kit session with injected anomalous pattern → flag
  emitted once (dedup); no block on capture path.

## Definition of done

Master §9 subset: plugin consumed by both kits; AUC gate in CI; events
schema-validated.