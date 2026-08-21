# Election-Day Rehearsal Report

> FIXTURE: This is a synthetic report generated from `report-template.md.tmpl` using stub data.
> It is NOT a real rehearsal run. Use it to validate the template renders correctly.

Date: 2026-08-21
Cluster context: eel-local-fixture
Prometheus: http://prometheus-fixture:9090
Base URL: http://localhost:8080
Mode: fixture

## 1. Cluster Snapshot

Topology: 2 sites (primary + mirror) + 5 edge regions (Nairobi, Mombasa, Kisumu, Nakuru, Eldoret).

```
NAME            STATUS   ROLES    AGE
node-primary-1  Ready    control  14d
node-primary-2  Ready    worker   14d
node-mirror-1   Ready    worker   14d
node-edge-nbo   Ready    worker   14d
node-edge-msa   Ready    worker   14d
```

Services ready: rts-svc 3/3, bvi-svc 3/3, portal-svc 2/2, verifier-rs 2/2, transparency-svc 2/2.

## 2. Load Profile

Scenario: `loadtest/scenarios/election-day.yaml` at scale 1.0.

| Phase | Duration | BVI rps | RTS concurrent | Portal rps | Kafka ev/s |
|---|---|---|---|---|---|
| registration-open | 60 min | 200 | 0 | 500 | 5,000 |
| polling-open | 120 min | 5,000 | 10,000 | 5,000 | 25,000 |
| morning-peak | 240 min | 18,000 | 60,000 | 1,389 | 100,000 |
| mid-day | 180 min | 4,000 | 8,000 | 800 | 15,000 |
| evening-peak | 120 min | 18,000 | 60,000 | 1,389 | 100,000 |
| close | 60 min | 500 | 5,000 | 2,000 | 30,000 |
| results-flood | 360 min | 100 | 60,000 | 5,000 | 100,000 |
| petition-window | 120 min | 0 | 500 | 1,000 | 2,000 |

Total kits: 65,000 (22% legacy Android shim). Telco impairment: 3-7% packet loss, satellite fallback.

## 3. SLA Table

| Metric | Measured | Target | Pass/Fail |
|---|---|---|---|
| bvi_auth_p95_latency | 148 ms | 200 ms | PASS |
| bvi_auth_p99_latency | 312 ms | 500 ms | PASS |
| bvi_throughput | 18,420 req/s | 18,000 req/s | PASS |
| rts_submit_p95_latency | 421 ms | 500 ms | PASS |
| rts_submit_p99_latency | 887 ms | 1,000 ms | PASS |
| rts_concurrent | 60,312 | 60,000 | PASS |
| write_path_p99_latency | 22 ms | 25 ms | PASS |
| portal_p95_latency | 161 ms | 200 ms | PASS |
| portal_p99_latency | 389 ms | 500 ms | PASS |
| portal_viewers_per_hour | 1,421 req/s (5.1M/hr) | 1,389 req/s (5M/hr) | PASS |
| verifier_concurrent | 10,240,000 | 10,000,000 | PASS |
| checkins_per_90min | 252,100 | 250,000 | PASS |
| kafka_events_per_sec | 101,200 | 100,000 | PASS |
| error_rate | 0.021% | < 0.1% | PASS |
| proof_generation_p99 | 8,200 ms | 30,000 ms | PASS |
| witness_sth_agreement | 5/5 | 5/5 | PASS |

All 16 SLAs: PASS.

## 4. Witness Cross-Check

Channels checked: Government web, GitHub mirror, IPFS-KE, Gazette, KBC.

| Channel | STH hash | Status |
|---|---|---|
| gov-web | sha256:abc123fixture | AGREE |
| github-mirror | sha256:abc123fixture | AGREE |
| ipfs-ke | sha256:abc123fixture | AGREE |
| gazette | sha256:abc123fixture | AGREE |
| kbc | sha256:abc123fixture | AGREE |

All 5 channels agree. Witness cross-check: PASS.

## 5. Bundle Repro Verdict

Command: `bash reproducibility/make-reproduce/scripts/reproduce.sh`

Result: PASS

Digest comparison:
```
sha256sum expected: deadbeef1234fixture000000000000000000000000000000000000000000000
sha256sum produced: deadbeef1234fixture000000000000000000000000000000000000000000000
```

Bit-identical: YES.

## 6. Verifier Output

Command: `verifier-rs verify-all --endpoint http://localhost:8080/transparency/proofs`

Result: PASS

Proofs verified: 65,000 (one per kit)
Proof failures: 0

## 7. Incidents

| Timestamp (UTC) | Severity | Description | Resolution |
|---|---|---|---|
| (none) | - | - | - |

## 8. Sign-Off

Rehearsal outcome: PASS (fixture run)

Signed by: eel-fixture-generator
Tlog meta-event: (not recorded for fixture runs)
