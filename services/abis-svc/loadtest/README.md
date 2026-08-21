# abis-svc Load Tests

Load tests for `abis-svc` validating ADR-0018 SLA targets.

## SLA Table

| Metric | Target | Fail Threshold | Script |
|---|---|---|---|
| Enrolment ingest p95 | < 1s | > 1000ms | `k6/enrolment-throughput.js` |
| Enrolment dedup p99 | < 8s | > 8000ms | `k6/enrolment-throughput.js` |
| 1:N match p95 | < 5s | > 5000ms | `k6/match-1n.js` |
| 1:N match p99 | < 8s | > 8000ms | `k6/match-1n.js` |
| 1:1 match p95 | < 100ms | > 100ms | `k6/match-11.js` |
| 1:1 match p99 | < 200ms | > 200ms | `k6/match-11.js` |
| Kit enrolment p95 | < 3s | > 3000ms | `k6/kit-enrolment.js` |
| Error rate | < 1% | > 1% | all |
| FMR | <= 0.001% | > 0.001% | observability |
| FNMR | <= 0.1% | > 0.1% | observability |

## Running

Prerequisites: [k6 >= 0.46](https://k6.io/docs/getting-started/installation/) and a running `abis-api` on `:8080`.

```
# Quick helper
./run-locally.sh match-11

# Direct k6 with overrides
k6 run --env ABIS_URL=http://localhost:8080 k6/enrolment-throughput.js

# Gatling (requires JVM + gatling runner)
gatling.sh -s abis.AbisScenarios
```

### Scenario descriptions

- `k6/enrolment-throughput.js`: Ramp to 10k enrolments/hour over 5 minutes, sustain 20 minutes.
- `k6/match-1n.js`: 100 concurrent VUs running 1:N searches against the gallery.
- `k6/match-11.js`: 500 rps constant-arrival 1:1 match for 15 minutes.
- `k6/kit-enrolment.js`: 500 concurrent kit enrolments with attestation header.
- `gatling/AbisScenarios.scala`: All four scenarios combined for 30 minutes.
- `scenarios/election-day.yaml`: Multi-phase 12-hour election-day workload profile definition.

### Synthetic fixtures

`fixtures/synthetic-templates/templates.jsonl` contains 100 deterministic template records (SHA-256-derived base64 blobs, no real biometric data).

### Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `ABIS_URL` | `http://localhost:8080` | Base URL of abis-api |
| `ATTEST_TOKEN` | `dev-attest-placeholder` | Attestation header value |
| `K6_OPTS` | (empty) | Extra k6 flags |
| `SKIP_K6_INSPECT` | (unset) | Set to skip k6 version check |
