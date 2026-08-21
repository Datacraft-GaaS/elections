# 5x Load Extension (WP-6.4)

5x peak write-path load test. Validates ADR-0018: P99 25 ms end-to-end at 250k writes/s sustained; Kafka 500k events/s with back-pressure headroom; regression gate against the 1x baseline from WP-3.10.

## Targets (5x Appendix-A)

| Metric | 1x Baseline | 5x Target |
|---|---|---|
| BVI req/s | 18,000 | 90,000 |
| RTS concurrent | 60,000 | 300,000 |
| Portal viewers/hr | 5,000,000 | 25,000,000 |
| Verifier concurrent | 10,000,000 | 50,000,000 |
| Kafka events/s | 100,000 | 500,000 |
| Write-path P99 | 25 ms | 25 ms (must hold) |

## Regression gate

After a 5x run, `regression-gate.sh` diffs the P99 numbers against the 1x baseline stored in `loadtest/profiles/local.yaml`. No metric may regress by more than 10% (ADR-0018). Results are written to `regression-report.json`.

## Running

```sh
# Dry run (syntax check only):
EEL_5X_MODE=dry bash verification/load-5x/regression-gate.sh

# Live 5x k6 run (requires pre-prod cluster):
EEL_BASE_URL=https://api.eel-preprod \
EEL_SCALING_FACTOR=5.0 \
EEL_KIT_COUNT=325000 \
  k6 run verification/load-5x/k6/5x/rts-submit.js

# Kafka producer (500k events/s for 60 s):
EEL_KAFKA_BROKERS=kafka.eel-preprod:9092 \
  bash verification/load-5x/kafka-100k/producer.sh

# Regression gate (after run):
EEL_BASELINE_PROFILE=loadtest/profiles/local.yaml \
EEL_5X_RESULTS=verification/reports/5x-results-YYYY-MM-DD.json \
  bash verification/load-5x/regression-gate.sh
```

## Files

| File | Purpose |
|---|---|
| profiles/5x.yaml | 5x scale profile extending loadtest/profiles/local.yaml |
| k6/5x/rts-submit.js | k6 RTS script at 5x (300k concurrent) |
| k6/5x/bvi-auth.js | k6 BVI script at 5x (90k req/s) |
| k6/5x/portal-view.js | k6 portal script at 5x (25M/hr) |
| k6/5x/verifier-lookup.js | k6 verifier script at 5x (50M concurrent) |
| gatling/5xSimulation.scala | Gatling RTS + BVI combined at 5x |
| kafka-100k/producer.sh | kcat producer at 500k events/s for 60 s |
| regression-gate.sh | Diff 5x P99 vs 1x baseline; assert < 10% regression |
| report-template.md.tmpl | 5x report skeleton |
| fixtures/local-5x-report.md | Synthetic fixture (marked) |
| tests/test_scripts.sh | Syntax + k6 inspect + Gatling sanity checks |
