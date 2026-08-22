# Backup restore drill: quarterly integrity verification

## Purpose

Prove that backups are restorable within the RPO/RTO targets before they are needed. A backup that cannot be restored is not a backup. This quarterly drill restores a recent pgBackRest backup and a Velero resource backup into a scratch namespace, verifies data integrity, and records the result in the transparency log.

## Prerequisites

- `pgbackrest` installed and configured at `/etc/pgbackrest/pgbackrest.conf` on the ops jump host.
- `velero` CLI installed and authenticated to the cluster.
- A scratch namespace `eel-drill` available (created and destroyed by this drill).
- An incident ticket open as `<TICKET>` (drill tickets are pre-approved).
- Database Lead and Site Reliability Lead present to witness the drill.

## Commands

### Step 1: confirm backup recency

```bash
pgbackrest --stanza=eel info
# confirm the latest full backup is within the RPO window (default 15 min)
velero backup get | head -20
# confirm a recent successful Velero backup exists
```

### Step 2: restore the pgBackRest backup to a scratch instance

```bash
# restore the latest backup into a temporary data directory:
pgbackrest --stanza=eel --type=time \
  --target-time="$(date -u -d '10 minutes ago' +%Y-%m-%dT%H:%M:%SZ)" \
  --target-action=promote restore
# start a throwaway postgres on the restored data dir:
pg_ctl -D /var/lib/pgbackrest/restore -o "-p 55432" start
```

### Step 3: verify restored data integrity

```bash
psql -p 55432 -d eel -c "SELECT count(*) FROM voters;"
# compare with the production count recorded at backup time
psql -p 55432 -d eel -c "SELECT max(updated_at) FROM events;"
# confirm the latest event timestamp is within the RPO window
```

### Step 4: restore a Velero resource backup

```bash
velero restore create --from-backup <BACKUP_NAME> \
  --namespace-mappings eel-system:eel-drill
velero restore get | grep <BACKUP_NAME>
# wait for status Phase=Completed
```

### Step 5: verify restored resources

```bash
kubectl get pods -n eel-drill
# confirm the restored workloads reach Running
kubectl get configmaps,secrets -n eel-drill
# confirm expected resources are present
```

### Step 6: tear down the scratch namespace

```bash
kubectl delete namespace eel-drill
pg_ctl -D /var/lib/pgbackrest/restore stop
rm -rf /var/lib/pgbackrest/restore
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.backup-restore-drill.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"pg_restore_ok\":true,\"velero_restore_ok\":true,\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Restored `voters` count matches the production count at backup time.
2. Restored `events` max timestamp is within the RPO window.
3. Velero restore reaches `Phase=Completed` and workloads reach `Running`.
4. Scratch namespace is fully removed.
5. Transparency log contains the drill-completed leaf.

## Rollback procedure

A failed restore is not rolled back; it is a finding. Record the failure in the ticket, escalate to the Database Lead, and schedule a corrective action. Do not delete the failing backup until the root cause is understood. If the scratch namespace cannot be removed, delete it manually with `kubectl delete namespace eel-drill --force --grace-period=0`.

## Escalation contacts

- Database Lead (on-call via PagerDuty rotation)
- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) if RPO/RTO targets are missed

## Estimated duration

45 to 90 minutes.

## Chaos-drill reference

This is a scheduled drill, not a chaos scenario. It exercises the same restore paths as `docs/runbooks/pgbackrest-pitr.md` and `docs/runbooks/velero-restore.md`.
