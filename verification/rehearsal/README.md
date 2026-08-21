# Election-Day Rehearsal (WP-6.1)

Full pre-production topology rehearsal for election day. Run in CI/pre-prod; local execution deferred because it requires a live cluster.

## Topology

- 2 sites (primary + mirror) with cross-site synchronous WAL replication
- 5 edge regions (Nairobi, Mombasa, Kisumu, Nakuru, Eldoret)
- Full T1/T2 service complement per `docs/adr/0020-service-topology.md`

## Scale targets (Appendix-A SLAs)

| Metric | Target |
|---|---|
| KIEMS kits | 65,000 |
| RTS concurrent | 60,000 |
| BVI throughput | 18,000 req/s |
| Portal viewers | 5,000,000 per hour |
| Polling-station check-ins | 250,000 per 90-min window |
| Kit verifier concurrent | 10,000,000 |
| Kafka ingest | 100,000 events/s |
| Write-path P99 (ADR-0018) | 25 ms end-to-end |

Every Appendix-A numeric SLA must be met. The rehearsal also validates:
- Witness cross-check: all 5 channels agree on the latest Signed Tree Head (STH)
- Declaration, portal, verifier, and bundle consistency after results close
- Reproducible bundle via `reproducibility/make-reproduce/scripts/reproduce.sh`
- Zero proof failures from `verifier-rs`

## Orchestrator stages

1. Verify cluster reachable
2. Toggle election-day-mode ON
3. Run `loadtest/scenarios/election-day.yaml` at scale 1.0
4. Collect Prometheus metrics for every SLA
5. Run witness cross-check
6. Rebuild reproducible bundle
7. Run `verifier-rs` on published proofs
8. Render report from `report-template.md.tmpl`

## Running

```sh
# Dry run (no cluster required):
EEL_REHEARSAL_MODE=dry bash verification/rehearsal/orchestrator.sh

# Full run (requires pre-prod cluster):
EEL_CLUSTER_CONTEXT=eel-preprod \
EEL_PROMETHEUS_URL=http://prometheus.eel-preprod:9090 \
EEL_BASE_URL=https://api.eel-preprod \
bash verification/rehearsal/orchestrator.sh
```

## Report

`fixtures/local-rehearsal-report.md` is a synthetic fixture rendered from the template. The CI run writes the real report to `verification/reports/rehearsal-YYYY-MM-DD.md`.

## Files

| File | Purpose |
|---|---|
| orchestrator.sh | Top-level driver |
| sla-manifest.yaml | One row per Appendix-A SLA with Prometheus query |
| witness-crosscheck.sh | Pings all 5 witness channels |
| report-template.md.tmpl | Report skeleton for envsubst rendering |
| fixtures/local-rehearsal-report.md | Synthetic fixture (marked) |
| tests/test_orchestrator.sh | Bats-style dry-run tests |
