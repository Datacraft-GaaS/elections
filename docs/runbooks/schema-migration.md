# Schema migration

## Purpose

Apply a database schema migration to a service's datastore. Migrations are versioned and applied in order; the goal is to apply a migration safely, verify it, and roll back cleanly if it fails. This runbook covers non-SpiceDB schema changes (Postgres, OpenSearch, MinIO layout).

## Prerequisites

- The migration files for the target service available (sqlx migrations, Django migrations, Ecto migrations).
- A backup of the affected datastore taken before the migration.
- An incident ticket open as `<TICKET>` (migration tickets are pre-approved).
- Database Lead present to witness the migration.

## Commands

### Step 1: confirm the current schema version

```bash
# list applied migrations and confirm the target is the next in sequence:
# sqlx: SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1;
# Django: python manage.py showmigrations
# Ecto: mix ecto.migrations
```

### Step 2: back up the datastore

```bash
# take a point-in-time backup before applying:
# Postgres: pg_dump or pgBackRest backup
# OpenSearch: snapshot to repository
# MinIO: versioned bucket copy
# record the backup reference as <BACKUP>
```

### Step 3: apply the migration

```bash
# apply the migration in the target environment:
# sqlx: sqlx migrate run
# Django: python manage.py migrate
# Ecto: mix ecto.migrate
# run in a maintenance window; the service may be read-only during the change
```

### Step 4: verify the migration

```bash
# confirm the schema version advanced and the migration is recorded:
# sqlx: SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1;
# confirm the service health endpoint returns "db":"ok":
kubectl exec -n eel-system deploy/<svc> -- \
  wget -qO- http://localhost:8080/health | grep '"db":"ok"'
```

### Step 5: run smoke checks

```bash
# exercise the affected read and write paths:
# run the service's conformance or smoke suite against the migrated schema
# confirm no new errors in the service logs
```

### Step 6: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.schema-migration.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"service\":\"<SVC>\",\"migration\":\"<VERSION>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. The schema version advanced to the target.
2. The service health endpoint returns `"db":"ok"`.
3. Smoke checks pass against the migrated schema.
4. No data loss (compare row counts before and after).
5. Transparency log contains the migration-completed leaf.

## Rollback procedure

If a migration fails or corrupts data, restore from the pre-migration backup `<BACKUP>` and re-apply any earlier migrations. For forward-only migrations, do not attempt to reverse the schema; restore the datastore instead. Confirm with the Database Lead before any restore.

## Escalation contacts

- Database Lead (on-call via PagerDuty rotation)
- Site Reliability Lead (on-call via PagerDuty rotation)
- Service owner for the affected service (contact via war room bridge)

## Estimated duration

15 to 45 minutes depending on migration size and datastore volume.

## Chaos-drill reference

This is a scheduled procedure, not a chaos scenario. It is related to `docs/runbooks/backup-restore-drill.md` (restore path) and `docs/runbooks/pg-cluster-failover.md` (primary switchover).
