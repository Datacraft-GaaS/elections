# DR Drill Report

**Drill date**: 2026-08-21
**Ticket**: DRILL-20260821
**Environment**: pre-prod (synthetic data, no PII)
**Operator(s)**: Alice K., Bob M.

## Summary

| Stage | Result | Elapsed (s) | Notes |
|---|---|---|---|
| Primary-site loss | PASS | 1423 | Patroni promoted in 8 min 12 s |
| Edge region loss | PASS | 612 | Region: nairobi, rerouted via mombasa |
| pgBackRest PITR | PASS | 834 | Stanza: eel, target time 2026-08-21T09:00:00Z |
| Velero restore | PASS | 412 | Namespace: eel-system, 12 pods running |
| Kopia restore | PASS | 290 | Snapshot: abc12345, 4 821 files, hash chain OK |
| Tlog attestation | PASS | 12 | Leaf index 42001, inclusion proof confirmed |

## Per-service RTO/RPO

| Service | Tier | Target RTO (min) | Measured RTO (min) | Target RPO (min) | Measured RPO (min) | Pass |
|---|---|---|---|---|---|---|
| rts-svc | T1-election-critical | 30 | 26 | 5 | 3 | YES |
| bvr-svc | T1-cycle | 60 | 48 | 15 | 11 | YES |
| bvi-svc | T1-cycle | 60 | 51 | 15 | 9 | YES |
| ecc-svc | T1-cycle | 60 | 44 | 15 | 12 | YES |
| transparency-svc | T1-cycle | 60 | 38 | 15 | 8 | YES |
| abis-svc | T1-cycle | 60 | 55 | 15 | 14 | YES |
| portal-svc | T2 | 240 | 72 | 60 | 28 | YES |

## Backup restore verification

| Tool | Stanza / Backup | Restore target | Verification query result | Pass |
|---|---|---|---|---|
| pgBackRest | eel | 2026-08-21T09:00:00Z | row_count: 1 847 | YES |
| Velero | eel-system-20260821-030000 | eel-system | Running pods: 12 | YES |
| Kopia | abc12345 | /tmp/kopia-restore | Files: 4 821, integrity verified | YES |

## Transparency log attestation

- Leaf type: `eel.dr.restore_completed.v1` (meta-events domain)
- Leaf hash: `a3f1c2d4e5b6a7f8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2`
- Leaf index: `42001`
- Inclusion proof: confirmed
- Proof latency (s): 12

## Deviations and action items

None. All SLOs met. Next drill scheduled for 2026-09-21 in pre-prod.

## Sign-off

| Role | Name | Signature |
|---|---|---|
| Ops lead | Alice K. | |
| Security officer | Bob M. | |
| Drill conductor | Carol N. | |

*Rendered from `verification/dr-drill/report-template.md.tmpl` with dry-run data.*
