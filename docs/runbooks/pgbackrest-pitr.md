# pgBackRest point-in-time restore (PITR)

## Purpose

Restore a PostgreSQL cluster to a specific point in time using pgBackRest, either as a drill to verify backup integrity or as a disaster recovery operation after data corruption. Drill restores target a non-production environment; disaster restores may target a standby or a fresh node.

## Prerequisites

- `pgbackrest` installed and configured at `/etc/pgbackrest/pgbackrest.conf`.
- The stanza name is `eel` (verify with `pgbackrest info`).
- The target recovery timestamp is known and noted as `<TARGET_TIME>` in ISO 8601 format (e.g., `2026-08-21T10:30:00+00`).
- For a drill: a separate Postgres instance (drill host) is available and stopped.
- For a disaster: the target node is stopped and Patroni is paused.
- An incident ticket open as `<TICKET>`.

## Commands

### Step 1: list available backups and WAL

```bash
pgbackrest --stanza=eel info
# review backup sets and WAL archive coverage
# confirm the target time falls within the WAL archive range shown
```

### Step 2: pause Patroni (disaster only, not for drills)

```bash
patronictl -c /etc/patroni/primary.yaml pause
# prevents Patroni from interfering with the restore
```

### Step 3: stop PostgreSQL on the target node

```bash
systemctl stop postgresql
# or on Kubernetes:
kubectl scale statefulset eel-pg -n eel-system --replicas=0
```

### Step 4: clear data directory

```bash
# confirm you are on the correct host before running this:
hostname && echo "Data dir: $PGDATA"
rm -rf "$PGDATA"/*
```

### Step 5: run pgBackRest restore with PITR target

```bash
pgbackrest --stanza=eel restore \
  --recovery-option="recovery_target_time=<TARGET_TIME>" \
  --recovery-option="recovery_target_action=promote" \
  --log-level-console=info
# this will restore the base backup and replay WAL up to the target time
# monitor progress in /var/log/pgbackrest/
```

### Step 6: start PostgreSQL and verify recovery

```bash
systemctl start postgresql
# or on Kubernetes:
kubectl scale statefulset eel-pg -n eel-system --replicas=1
# check recovery completion:
psql -U postgres -c "SELECT pg_is_in_recovery(), now();"
# pg_is_in_recovery must return 'f' (false) after promotion
```

### Step 7: verify row counts

```bash
psql -U postgres -d eel -c "SELECT COUNT(*) FROM voters;"
psql -U postgres -d eel -c "SELECT COUNT(*) FROM tally_events;"
# compare against pre-incident counts from monitoring snapshots
```

### Step 8: run pgBackRest check post-restore

```bash
pgbackrest --stanza=eel check
# must complete without errors
```

### Step 9: resume Patroni (disaster only)

```bash
patronictl -c /etc/patroni/primary.yaml resume
patronictl -c /etc/patroni/primary.yaml list
# verify the restored node becomes the Leader or a healthy Replica
```

### Step 10: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.pgbackrest-pitr.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"target_time\":\"<TARGET_TIME>\",\"drill\":\"<true|false>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `pg_is_in_recovery()` returns false (cluster is promoted).
2. Row counts match expected values for the target time.
3. `pgbackrest check` exits 0.
4. Patroni shows the node as Leader or Replica (no errors).
5. Application health endpoints return `"db":"ok"`.

## Rollback procedure

For a drill: shut down the drill instance; the production cluster is unaffected.

For a disaster: if the restore target time was wrong or row counts do not match, stop PostgreSQL, clear the data directory again, and repeat from step 5 with a different target time. Document each attempt in the incident ticket.

## Escalation contacts

- Database Lead (on-call via PagerDuty rotation)
- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge)

## Estimated duration

30 to 60 minutes depending on backup size and WAL replay volume.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-pg-primary-kill-patroni-failover.yaml`

Drill schedule: quarterly, per `docs/runbooks/backup-restore-drill.md`.
