# platform/pgbackrest/

pgBackRest configuration for PostgreSQL WAL archiving, continuous backup, and point-in-time recovery (PITR) for the eel IEMS.

## Contents

| File | Description |
|---|---|
| `pgbackrest.conf` | pgBackRest global configuration (stanzas, repo paths, retention) |
| `postgresql-archive.conf` | PostgreSQL `archive_command` and `restore_command` settings |
| `backup-schedule-cronjob.yaml` | Kubernetes `CronJob` for full and incremental backups |
| `pitr-restore-procedure.md` | Step-by-step PITR restore runbook |

## Backup schedule

| Type | Schedule | Retention |
|---|---|---|
| Full backup | Sunday 01:00 UTC | 4 weeks |
| Incremental | Daily 01:00 UTC (Mon-Sat) | 30 days |
| WAL archiving | Continuous | 30 days |

## Stanzas

Each PostgreSQL cluster has a named stanza in `pgbackrest.conf`:

| Stanza | Cluster | Repo |
|---|---|---|
| `voter-db` | voter register primary | `repo1` (MinIO primary), `repo2` (MinIO mirror) |
| `rts-db` | rts-svc results database | `repo1`, `repo2` |
| `audit-db` | transparency-svc audit log | `repo1`, `repo2` |

## Apply configuration

```bash
# Apply backup schedule CronJob:
kubectl apply -f platform/pgbackrest/backup-schedule-cronjob.yaml

# Trigger a manual full backup:
kubectl create job --from=cronjob/pgbackrest-full-backup manual-full-backup-$(date +%Y%m%d)
```

## PITR restore

Follow `pitr-restore-procedure.md` for step-by-step instructions. The short form:

```bash
# Stop PostgreSQL on the target host:
systemctl stop postgresql

# Restore to a point in time:
pgbackrest --stanza=rts-db \
  --type=time "--target=2026-08-15T14:32:00Z" \
  --target-action=promote restore
```

## Verify backups

```bash
pgbackrest --stanza=rts-db info
# Confirm: backup=full, status=ok, WAL archive present
```

## Operator responsibilities

- Monitor `pgbackrest_last_backup_age_seconds` in Grafana. Alert fires if > 26 hours.
- Test PITR quarterly: restore to a scratch cluster, verify row counts and WAL replay.
- Backup repos are on MinIO; ensure `repo1` and `repo2` are on separate physical Ceph pools.

## Related docs

- `platform/postgres/README.md`: PostgreSQL cluster definitions
- `platform/minio/README.md`: S3 backup repository
- `docs/runbooks/dr-failover.md`: DR failover uses PITR as part of the failover sequence
