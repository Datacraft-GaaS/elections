# 5x Load Test Report

> FIXTURE: Synthetic report. Not a real 5x run. Validates template structure only.

Date: 2026-08-21
Cluster context: eel-local-fixture
Scale factor: 5.0
Baseline profile: loadtest/profiles/local.yaml (WP-3.10 baseline)

## Summary

| | Value |
|---|---|
| Overall result | PASS |
| Regressions | 0 |
| BVI throughput achieved | 91,200 req/s (target 90k) |
| RTS peak concurrent | 301,400 (target 300k) |
| Kafka throughput | 503,800 events/s (target 500k) |
| Write-path P99 | 21 ms (ADR-0018 gate: 25 ms) |

## Per-Metric Table

| Metric | 1x Baseline | 5x Measured | Delta % | ADR-0018 Target | Pass/Fail |
|---|---|---|---|---|---|
| bvi_auth_p95_ms | 200 ms | 162 ms | -19.0% | 200 ms | PASS |
| bvi_auth_p99_ms | 500 ms | 341 ms | -31.8% | 500 ms | PASS |
| rts_submit_p95_ms | 500 ms | 412 ms | -17.6% | 500 ms | PASS |
| rts_submit_p99_ms | 1000 ms | 881 ms | -11.9% | 1000 ms | PASS |
| write_path_p99_ms | 25 ms | 21 ms | -16.0% | 25 ms | PASS |
| portal_p95_ms | 200 ms | 158 ms | -21.0% | 200 ms | PASS |
| portal_p99_ms | 500 ms | 372 ms | -25.6% | 500 ms | PASS |
| kafka_events_per_sec | 100k | 503,800 | +403.8% | 500k | PASS |

## Regression Gate

Regression threshold: 10% (ADR-0018).

Regressions detected: 0
Regression report: verification/reports/regression-report-2026-08-21.json

## Kafka Headroom

Target: 20% headroom above 500k events/s (= 600k events/s demonstrated).
Measured headroom rate: 621,000 events/s
Headroom status: PASS (124% of target 500k, 24% above 500k threshold)

## Incidents

| Timestamp (UTC) | Severity | Description | Resolution |
|---|---|---|---|
| (none) | - | - | - |

## Sign-Off

5x load test outcome: PASS (fixture)
