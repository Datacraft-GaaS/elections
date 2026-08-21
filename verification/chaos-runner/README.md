# Chaos Suite Runner (WP-6.3)

Composes all 34 chaos scenarios from `chaos/scenarios/` against the WP-6.1 election-day rehearsal load. No SLO breach beyond the degradation contract is permitted.

## Design

The runner applies scenarios in the order declared by `schedule.yaml`, which follows the election-day phase structure:

- Pod-kill scenarios: early in the rehearsal (polling-open phase) -- individual service resilience
- Site partition: mid-load (morning-peak) -- cross-site network split, tests Patroni fence + MirrorMaker2 lag
- Timing skew: late-load (evening-peak) -- HLC monotonicity under clock drift
- Security scenarios: any time -- cert expiry, SVID revocation, SpiceDB down, HSM unavailable
- IO + stress: mid-load -- disk-full, CPU/mem pressure
- Misc: spread across the day -- Kafka ISR shrink, Debezium pause, Valkey flush, DNS chaos

## Degradation contract

Each service has documented tolerances in `degradation-contract.yaml`. These mirror the platform/argocd degradation runbook from WP-5.10.

If any scenario causes Prometheus SLO burn to exceed the declared allowance, the runner:
1. Logs the breach with metric values
2. Kills the chaos scenario immediately
3. Writes a red report to `verification/reports/chaos-YYYY-MM-DD-red.md`
4. Exits non-zero

## Running

```sh
# Dry run (validates all scenario refs exist):
EEL_CHAOS_MODE=dry bash verification/chaos-runner/runner.sh

# Live run (alongside rehearsal orchestrator):
EEL_CLUSTER_CONTEXT=eel-preprod \
EEL_PROMETHEUS_URL=http://prometheus.eel-preprod:9090 \
EEL_REHEARSAL_START="$(date -u +%s)" \
bash verification/chaos-runner/runner.sh
```

## Files

| File | Purpose |
|---|---|
| scenarios-index.yaml | All 34 chaos scenarios with category/target/degradation |
| runner.sh | Driver: applies scenarios, watches SLO burn, aborts on breach |
| schedule.yaml | Canonical order and start-time offsets for T-0 rehearsal |
| degradation-contract.yaml | Per-service normal/degraded/read-only tolerances |
| report-template.md.tmpl | Chaos report skeleton |
| fixtures/local-chaos-report.md | Synthetic fixture (marked) |
| tests/test_runner.sh | Validates all scenario refs exist |
