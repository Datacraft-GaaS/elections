# loadtest

Cross-service election-day load test harness for eel. This directory contains the
rehearsal framework that stress-tests all 18 services together under realistic Kenya
election-day traffic patterns: 22M voters, 65,000 KIEMS kits, 46,229 polling stations,
bimodal morning and evening peaks, telco impairment profiles, and a results-flood phase.

## Relationship to per-service loadtest directories

Each service has its own `services/*/loadtest/` directory for unit-level load scenarios
targeting that service in isolation. This top-level `loadtest/` directory is different:
it runs cross-service scenarios that exercise the full request path from edge to database.

| Scope | Location | When to run |
|-------|----------|-------------|
| Per-service isolation | `services/<svc>/loadtest/` | On every PR that touches the service |
| Cross-service rehearsal | `loadtest/` (here) | Weekly against staging from month 2; mandatory before election day |

## Contents

| Directory | Description |
|-----------|-------------|
| `scenarios/` | YAML scenario definitions (election-day, petition-window, results-flood) |
| `k6/` | k6 JavaScript scripts for HTTP load generation |
| `gatling/` | Gatling Scala simulations for high-concurrency HTTP scenarios |
| `profiles/` | Scaling profiles: local (1/100 scale), staging (1x), pre-prod (5x) |
| `kit-simulator/` | Rust kit simulator: emulates 65k KIEMS kits with BVI, RTS, heartbeat |
| `reports/` | Historical load test reports and regression baselines |

## Quick start

```bash
# Local smoke test (1/100 scale, kind cluster)
k6 run loadtest/k6/portal-visitors.js \
  -e EEL_BASE_URL=http://localhost:8080 \
  -e EEL_KIT_COUNT=600

# Full election-day rehearsal on staging (1x scale)
cd loadtest
k6 run k6/rts-submit.js \
  -e EEL_BASE_URL=https://staging.eel.internal \
  -e EEL_KIT_COUNT=60000

# 5x stress run (pre-prod only)
K6_PROFILE=prod-dry-run k6 run k6/rts-submit.js \
  --config profiles/prod-dry-run.yaml

# Run all k6 scenarios in sequence
for script in loadtest/k6/*.js; do
  k6 run "$script" -e EEL_BASE_URL=https://staging.eel.internal
done
```

## Profiles

| Profile file | Scaling factor | Intended environment |
|--------------|---------------|----------------------|
| `profiles/local.yaml` | 0.01 (1/100) | kind cluster, developer machine |
| `profiles/staging.yaml` | 0.1 (1/10) | staging cluster |
| `profiles/pre-prod.yaml` | 1.0 (1x) | pre-prod cluster |
| `profiles/prod-dry-run.yaml` | 5.0 (5x) | pre-prod 5x stress run |

SLA targets from Appendix-A are multiplied by the profile's `scaling_factor` to get
local targets (e.g., BVI p95 raw 20,000ms becomes 200ms local floor).

## Config and environment

| Variable | Description |
|----------|-------------|
| `EEL_BASE_URL` | Base URL for all k6 HTTP requests |
| `EEL_KIT_COUNT` | Number of simulated kits (default: profile value) |
| `K6_PROFILE` | Profile name; maps to a file in `profiles/` |
| `GATLING_BASE_URL` | Base URL for Gatling simulations |

## Dependencies

- k6 >= 0.50 (`brew install k6` or from nixpkgs)
- Gatling 3.10+ (JVM 17; `build.sbt` in `gatling/`)
- Rust toolchain for kit-simulator (if building from source)
- A running eel deployment (local kind, staging, or pre-prod)

## Test and verify

```bash
# Smoke: portal visitors, local scale
k6 run loadtest/k6/portal-visitors.js \
  -e EEL_BASE_URL=http://localhost:8080 \
  -e EEL_KIT_COUNT=100
# Expected: all thresholds green, p95 < 500ms, error rate < 1%

# Check results threshold from a previous run
cat loadtest/reports/staging-election-day-$(date +%Y%m).json \
  | jq '.thresholds | to_entries[] | select(.value.ok == false)'
# Expected: empty (all thresholds passed)
```

## Related docs

- `loadtest/scenarios/README.md`: scenario format and available scenarios
- `loadtest/k6/README.md`: k6 script reference
- `loadtest/kit-simulator/README.md`: kit simulator usage
- `docs/plans/00-principles.md`: local SLA scaling table (Appendix-A mapping)
- `services/*/loadtest/`: per-service load tests

## Limitations

- The kit simulator does not emulate satellite fallback paths or legacy Android shim
  behavior; use the `garissa` scenario override for satellite testing.
- 5x stress runs require a pre-prod cluster with at least 3x normal node count; do not
  run on staging.
- Gatling simulations require JVM 17+ and take 2-3 minutes to JIT-compile before
  generating meaningful load; do not interrupt within the first 3 minutes.
