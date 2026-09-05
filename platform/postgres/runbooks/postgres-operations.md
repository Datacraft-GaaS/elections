# Runbook: PostgreSQL Operations

## Health Checks

```bash
# Cluster health
kubectl get clusters -n eel-postgres -o wide

# Pod health
kubectl get pods -n eel-postgres -o wide

# Replication status (primary → replicas)
kubectl exec -n eel-postgres deploy/eel-postgres-primary -- psql -c "SELECT * FROM pg_stat_replication;"

# Audit chain verification (database audit events must have auditId/proofHash)
# Check audit_service writes events referencing audit table
```

## Backup Verification

```bash
# Check pgBackRest schedule
kubectl get cronjobs -n eel-backup
# Verify latest backup exists at DR site
# Reference: `platform/pgbackrest/backup-schedule-cronjob.yaml`
```

## Common Issues

| Issue | Diagnosis | Resolution |
|---|---|---|
| Primary down | `kubectl describe cluster` | Promote DR replica |
| Replica lag > 5 min | `SELECT lag FROM pg_stat_replication;` | Check WAL archive, restart replication |
| Audit events missing | Check audit_service connection to DB | Verify audit_service deployment |
| Backup failing | `kubectl logs -n eel-backup` | Check storage access, credentials |

## References

- `platform/postgres/README.md` (node pools: audit, voter, consensus)
- `platform/pgbackrest/README.md` (backup schedule)
- M5-SMS-USB-MANIFEST (RPO/RTO targets)
- Elections Act §34 (data integrity during DR)
