# Chaos Suite Report

> FIXTURE: Synthetic report. Not a real chaos run. Validates template structure only.

Date: 2026-08-21
Cluster context: eel-local-fixture
Rehearsal start: 1724198400
Mode: fixture

## Summary

Scenarios applied: 34
Scenarios completed without breach: 34
SLO breaches: 0
Overall result: PASS

## Per-Scenario Results

| Scenario | Category | Applied at | Duration | SLO Burn | Status |
|---|---|---|---|---|---|
| eel-pod-kill-training-svc | pod-kill | T+5min | 60s | 0.001 | PASS |
| eel-pod-kill-crms-svc | pod-kill | T+10min | 60s | 0.001 | PASS |
| eel-pod-kill-governance-svc | pod-kill | T+15min | 60s | 0.002 | PASS |
| eel-pod-kill-support-admin-svc | pod-kill | T+20min | 60s | 0.001 | PASS |
| eel-pod-kill-inventory-svc | pod-kill | T+25min | 60s | 0.001 | PASS |
| eel-pod-kill-ecc-svc | pod-kill | T+65min | 60s | 0.003 | PASS |
| eel-pod-kill-queue-svc | pod-kill | T+75min | 90s | 0.004 | PASS |
| eel-pod-kill-field-mobile-svc | pod-kill | T+85min | 90s | 0.002 | PASS |
| eel-pod-kill-boundary-svc | pod-kill | T+95min | 60s | 0.008 | PASS |
| eel-pod-kill-ai-platform-svc | pod-kill | T+110min | 60s | 0.001 | PASS |
| eel-pod-kill-biometric-edge-svc | pod-kill | T+120min | 90s | 0.003 | PASS |
| eel-valkey-flush | misc | T+130min | 120s | 0.009 | PASS |
| eel-pod-kill-portal-svc | pod-kill | T+185min | 30s | 0.011 | PASS |
| eel-pod-kill-bvr-svc | pod-kill | T+195min | 60s | 0.004 | PASS |
| eel-pod-kill-bvi-svc | pod-kill | T+210min | 28s | 0.031 | PASS |
| eel-pod-kill-rts-svc | pod-kill | T+230min | 52s | 0.042 | PASS |
| eel-pod-kill-vprs-svc | pod-kill | T+250min | 60s | 0.004 | PASS |
| eel-pod-kill-abis-svc | pod-kill | T+265min | 60s | 0.003 | PASS |
| eel-pod-kill-transparency-svc | pod-kill | T+280min | 60s | 0.012 | PASS |
| eel-network-loss-30pct-2s-latency | misc | T+300min | 300s | 0.062 | PASS |
| eel-site-partition-primary-mirror | partition | T+320min | 300s | 0.089 | PASS |
| eel-stress-chaos-cpu-mem | stress | T+355min | 300s | 0.071 | PASS |
| eel-io-chaos-disk-full | io | T+375min | 180s | 0.088 | PASS |
| eel-debezium-pause | misc | T+390min | 300s | 0.021 | PASS |
| eel-dns-chaos | misc | T+405min | 180s | 0.044 | PASS |
| eel-spicedb-down | security | T+430min | 120s | 0.062 | PASS |
| eel-hsm-unavailable | security | T+450min | 120s | 0.091 | PASS |
| eel-kafka-isr-shrink | misc | T+470min | 600s | 0.112 | PASS |
| eel-minio-site-loss | misc | T+490min | 300s | 0.038 | PASS |
| eel-sequencer-stall | misc | T+510min | 300s | 0.028 | PASS |
| eel-time-chaos-plusminus-10min | time | T+615min | 600s | 0.041 | PASS |
| eel-svid-revocation | security | T+730min | 300s | 0.044 | PASS |
| eel-cert-expiry-15min | security | T+750min | 900s | 0.031 | PASS |
| eel-pg-primary-kill-patroni-failover | misc | T+470min | 30s | 0.112 | PASS |

## SLO Burn Budget

All scenarios remained within their declared `allowed_slo_burn` envelope.
Maximum observed burn: 0.112 (eel-kafka-isr-shrink, allowance 0.20).

## Incidents

| Timestamp (UTC) | Scenario | Severity | Description | Resolution |
|---|---|---|---|---|
| (none) | - | - | - | - |

## Sign-Off

Chaos run outcome: PASS (fixture)

Log file: verification/reports/chaos-2026-08-21.jsonl
