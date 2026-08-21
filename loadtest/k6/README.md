# loadtest/k6

k6 JavaScript load test scripts for eel cross-service scenarios. Each script targets one
service endpoint or interaction pattern; they are composed together by the election-day
rehearsal run.

## Scripts

| Script | Service | Endpoint / pattern | Peak target |
|--------|---------|-------------------|-------------|
| `rts-submit.js` | `rts-svc` | 34A form submission (M08, M10) | 60k concurrent, p95 <= 500ms |
| `bvi-auth.js` | `bvi-svc` | Biometric 1:1 identification | 18k req/s, p95 <= 20s |
| `portal-visitors.js` | `portal-svc` | Public results page visitors | 5M/hr, p95 <= 2s |
| `personnel-checkins.js` | `bvr-svc` | Polling station check-in (M33) | 250k per 90min |
| `verifier-mobile.js` | `bvi-svc` verifier | Mobile verifier concurrent | 10M concurrent (M13) |

## Usage

```bash
# Run a single script at local scale
k6 run loadtest/k6/rts-submit.js \
  -e EEL_BASE_URL=http://localhost:8080 \
  -e EEL_KIT_COUNT=600

# Run at staging scale with the staging profile
k6 run loadtest/k6/rts-submit.js \
  --config loadtest/profiles/staging.yaml \
  -e EEL_BASE_URL=https://staging.eel.internal \
  -e EEL_KIT_COUNT=6000

# Run all scripts sequentially
for f in loadtest/k6/*.js; do
  k6 run "$f" \
    -e EEL_BASE_URL=http://localhost:8080 \
    -e EEL_KIT_COUNT=600
done

# Output results to InfluxDB for Grafana dashboard
k6 run loadtest/k6/rts-submit.js \
  -e EEL_BASE_URL=http://localhost:8080 \
  --out influxdb=http://influxdb.eel.internal:8086/k6
```

## Script conventions

All scripts follow these conventions:

- `EEL_BASE_URL` env var for the target URL.
- `EEL_KIT_COUNT` env var for the number of simulated kits (used to set VU counts).
- Thresholds named after Appendix-A SLA identifiers (e.g., `rts_p95_ms`, `bvi_auth_p99_ms`).
- Custom metrics use the `Trend` and `Counter` k6 types; metric names are prefixed by service.
- Scenarios use `ramping-vus` executor with a ramp-up, peak, and wind-down stage.

## Config and environment

| Variable | Description |
|----------|-------------|
| `EEL_BASE_URL` | HTTP base URL for all requests (no trailing slash) |
| `EEL_KIT_COUNT` | Simulated kit count; VU counts are derived as fractions of this |
| `EEL_SCENARIO` | Optional: path to a scenario YAML; scripts parse it if present |
| `EEL_AUTH_TOKEN` | Bearer token for authenticated endpoints (BVI, RTS, BVR) |

## Dependencies

- k6 >= 0.50
- A running eel deployment reachable at `EEL_BASE_URL`
- Optional: InfluxDB + Grafana for real-time metrics during a run

## Test and verify

```bash
# Dry-run (no actual HTTP requests; validates script syntax)
k6 run loadtest/k6/rts-submit.js --dry-run

# Local smoke test
k6 run loadtest/k6/portal-visitors.js \
  -e EEL_BASE_URL=http://localhost:8080 \
  -e EEL_KIT_COUNT=50
# Expected: all thresholds green in the summary output
```

## Related docs

- `loadtest/README.md`: harness overview
- `loadtest/scenarios/README.md`: scenario definitions driving VU counts
- `loadtest/profiles/README.md`: scaling profiles
- `loadtest/reports/README.md`: stored reports and baselines

## Limitations

- k6 scripts do not simulate the full CBOR wire format used by kit firmware; they use
  JSON equivalents. For CBOR-fidelity testing, use the kit-simulator.
- Virtual users in k6 share a single process; very high VU counts (>50k) require k6
  distributed mode with multiple k6 instances.
