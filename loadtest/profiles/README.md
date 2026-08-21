# loadtest/profiles

Scaling profiles for the eel load test harness. Each profile applies a `scaling_factor`
to the raw Appendix-A SLA values, mapping national election-day targets to the capacity
of a specific test environment.

## Purpose

Running at full 1x scale requires a production-equivalent cluster. Developer machines and
CI run at 1/100 scale. The profile system lets every script and simulation read the same
SLA values and scale them consistently without hard-coding environment-specific numbers
into scripts.

All k6 scripts and the kit simulator read the active profile (via `EEL_PROFILE` or
`--profile`) and multiply raw Appendix-A values by `scaling_factor` to produce local
targets.

## Profiles

| File | `scaling_factor` | Intended environment | BVI p95 target (raw: 20s) |
|------|-----------------|---------------------|--------------------------|
| `local.yaml` | 0.01 (1/100) | kind cluster, developer laptop | 200ms (floor applied) |
| `staging.yaml` | 0.1 (1/10) | staging cluster (3 nodes) | 2s |
| `pre-prod.yaml` | 1.0 (1x) | pre-prod cluster (prod-equivalent) | 20s |
| `prod-dry-run.yaml` | 5.0 (5x) | pre-prod 5x stress run | 100s ceiling |

The local profile applies a 50ms floor for all latency targets to avoid unrealistic
sub-millisecond thresholds on a local machine.

## Profile format

```yaml
scaling_factor: 0.01

# BVI biometric identification
bvi_auth_p95_ms: 200     # raw 20000ms * 0.01 = 200ms (floor applied)
bvi_auth_p99_ms: 500
bvi_req_per_sec: 1       # raw 100 rps

# RTS results transmission
rts_submit_p95_ms: 100
rts_submit_p99_ms: 250
rts_concurrent: 5        # raw 500

# ... (one key per Appendix-A SLA metric)
```

## Usage

```bash
# k6: pass profile via --config
k6 run loadtest/k6/rts-submit.js \
  --config loadtest/profiles/staging.yaml \
  -e EEL_BASE_URL=https://staging.eel.internal

# Kit simulator: pass via --profile flag
loadtest/kit-simulator/eel-kit-sim \
  --profile loadtest/profiles/local.yaml \
  --base-url http://localhost:8080

# Override a single threshold on the command line
k6 run loadtest/k6/rts-submit.js \
  --config loadtest/profiles/local.yaml \
  -e RTS_CONCURRENT=20
```

## Related docs

- `loadtest/README.md`: harness overview
- `loadtest/scenarios/README.md`: scenario files that provide the raw values
- `docs/plans/00-principles.md`: Appendix-A SLA source and the 1/100 local scaling rule

## Limitations

- Profiles do not validate that the test environment has sufficient resources; running
  `pre-prod.yaml` on staging will overload the staging cluster.
- The 5x `prod-dry-run.yaml` requires pre-prod cluster capacity planning; coordinate
  with the infrastructure team before scheduling.
- Profile updates that change SLA thresholds must be reviewed against ADR-0019 and the
  tender Appendix-A; do not relax thresholds without approval.
