# DR Drill (WP-6.2)

Verification harness for disaster-recovery failover and restore drills. Scripts are
designed to run in a pre-prod environment against the eel two-site topology. No
production traffic is affected. Every restore operation is attested to the
transparency log.

## RTO / RPO targets

| Tier | Service example | RTO | RPO |
|---|---|---|---|
| T1-election-critical | rts-svc | 30 min | 5 min |
| T1-cycle | bvr-svc, bvi-svc, ecc-svc, transparency-svc, abis-svc | 60 min | 15 min |
| T2 | portal-svc, vprs-svc, governance-svc, etc. | 4 h | 1 h |
| Edge region | nairobi, mombasa, kisumu, nakuru, eldoret | 15 min | n/a |

See `rto-rpo-matrix.yaml` for per-service detail.

## Drill sequence

1. `primary-loss.sh` - simulate primary-site loss; verify mirror promotion via Patroni.
2. `edge-loss.sh` - cordon one regional edge; verify anycast reroute within 15 min.
3. `restore-pgbackrest.sh` - PITR restore against synthetic basebackup fixture.
4. `restore-velero.sh` - Velero namespace restore against synthetic backup.
5. `restore-kopia.sh` - Kopia snapshot restore for MinIO buckets; assert hash chain.
6. `tlog-attest-restore.sh` - submit `eel.dr.restore_completed.v1` leaf to tlog; assert inclusion proof within 60 s.

## Dry mode

Set `EEL_DR_MODE=dry` to run all scripts without touching live infrastructure. Each
script exits 0 and emits a JSON stage summary to stdout.

## Prerequisites

- `eel-dr-failover` binary built: `cargo build --release -p eel-dr-failover`
- `kubectl`, `patronictl`, `pgbackrest`, `velero`, `kopia` on PATH
- `curl` or `grpcurl` for tlog submission
- Access to pre-prod kubeconfig at `~/.kube/config` or `EEL_KUBECONFIG`

## Fixtures

`fixtures/synthetic-basebackup/` contains a minimal pgBackRest-compatible stub for
dry-run and CI exercises. Real basebackup restores run against the live pgBackRest
stanza in pre-prod MinIO.

## References

- Runbook: `docs/runbooks/dr-failover-primary-to-mirror.md`
- Runbook: `docs/runbooks/pgbackrest-pitr.md`
- Runbook: `docs/runbooks/velero-restore.md`
- ADR-0022: Release process and DR posture
- ADR-0017: Transparency log
