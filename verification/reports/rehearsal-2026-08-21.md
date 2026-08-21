# Election-Day Rehearsal Report

Date: 2026-08-21
Cluster context: eel-preprod
Prometheus: http://prometheus.eel-preprod:9090
Base URL: https://api.eel-preprod
Mode: dry

## 1. Cluster Snapshot

Topology: 2 sites (primary + mirror) + 5 edge regions (Nairobi, Mombasa, Kisumu, Nakuru, Eldoret).

Node count, service versions, and readiness summary captured from:
```
kubectl -n eel-prod get nodes,deployments,statefulsets -o wide
```

Paste cluster snapshot here (auto-filled by orchestrator when running against live cluster).

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
| (dry run) | - | - | - |

## 4. Witness Cross-Check

Channels checked: Government web, GitHub mirror, IPFS-KE, Gazette, KBC.

| Channel | STH hash | Status |
|---|---|---|
| gov-web | (filled by witness-crosscheck.sh) | - |
| github-mirror | - | - |
| ipfs-ke | - | - |
| gazette | - | - |
| kbc | - | - |

All 5 channels must agree on the latest Signed Tree Head (STH). Disagreement = rehearsal FAIL.

## 5. Bundle Repro Verdict

Command: `bash reproducibility/make-reproduce/scripts/reproduce.sh`

Result: (PASS / FAIL)

Digest comparison:
```
sha256sum: (expected) = (produced)
```

## 6. Verifier Output

Command: `verifier-rs verify-all --endpoint https://api.eel-preprod/transparency/proofs`

Result: (PASS / FAIL)

Proofs verified: (count)
Proof failures: 0 (required)

## 7. Incidents

| Timestamp (UTC) | Severity | Description | Resolution |
|---|---|---|---|
| (none) | - | - | - |

## 8. Sign-Off

Rehearsal outcome: (PASS / FAIL)

Signed by: (eel coordinator + OIT)
Tlog meta-event: (tlog entry hash for G6 attestation)
