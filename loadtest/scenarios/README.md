# loadtest/scenarios

YAML scenario definitions for the eel cross-service load test harness. Each scenario
describes a full traffic pattern: phases, per-phase RPS/concurrency targets, telco
impairment profiles, and kit counts.

## Purpose

Scenarios are the authoritative source of truth for what a load test run represents.
k6 scripts and Gatling simulations read scenario YAML to configure their VU counts,
ramp curves, and thresholds. This separation lets the same script run multiple scenarios
without code changes.

## Contents

| File | Description |
|------|-------------|
| `election-day.yaml` | Full 12-hour polling day plus results and petition window. The primary rehearsal scenario. |

## election-day.yaml

The election-day scenario covers:

- 65,000 KIEMS kits (22% on legacy Android shim path)
- 30-second kit heartbeat interval
- 12 polling phases from 05:00-19:00 EAT: registration-open, polling-open, morning-peak,
  midday, afternoon, evening-peak, close, results-flood
- Per-phase BVI RPS, RTS concurrent connections, portal RPS, verifier concurrent,
  polling station check-ins, Kafka events/sec
- Telco impairment per phase: packet loss %, satellite fallback %, 3G/4G/LTE tech mix

Peak values (morning-peak phase, 1x scale):
- BVI: 18,000 req/s
- RTS: 60,000 concurrent
- Portal: 1,389 req/s (5M/hr)
- Verifier: 10M concurrent (M13)
- Kafka: 100,000 events/s

## Usage

```bash
# k6 scripts read scenario via EEL_SCENARIO env var
k6 run loadtest/k6/rts-submit.js \
  -e EEL_SCENARIO=loadtest/scenarios/election-day.yaml \
  -e EEL_BASE_URL=https://staging.eel.internal

# Kit simulator reads scenario directly
loadtest/kit-simulator/eel-kit-sim \
  --scenario loadtest/scenarios/election-day.yaml \
  --profile loadtest/profiles/staging.yaml \
  --base-url https://staging.eel.internal
```

## Adding a new scenario

1. Copy `election-day.yaml` to `<new-scenario>.yaml`.
2. Adjust phases, RPS values, and telco profile.
3. Update `loadtest/README.md` contents table.
4. Add a CI entry in `.github/workflows/loadtest.yaml` for the new scenario.

## Related docs

- `loadtest/README.md`: harness overview
- `loadtest/profiles/README.md`: scaling profiles applied on top of scenario values
- `loadtest/k6/README.md`: k6 scripts that consume scenarios

## Limitations

- Scenario YAML is not formally schema-validated; an invalid field is silently ignored by
  some scripts. Add schema validation before relying on custom scenarios in pre-prod.
- The single `election-day.yaml` covers the 2027 general election pattern; future elections
  with different kit counts or SLA targets will need a new scenario file.
