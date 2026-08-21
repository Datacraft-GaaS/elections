# platform/postgres/

PostgreSQL 17 cluster configuration for the eel IEMS. Each Tier 1 service owns a dedicated PostgreSQL cluster managed by the CloudNativePG operator.

## Contents

| Path | Description |
|---|---|
| `local/` | Kustomize overlay for kind (single-instance, testcontainers-compatible) |

Per-service cluster definitions live in `services/<svc>/deploy/overlays/*/postgres-cluster.yaml`.

## Cluster model

| Service | Cluster name | Instances | Data class |
|---|---|---|---|
| rts-svc | `rts-db` | 3 (primary + 2 replicas) | SECRET |
| bvr-svc | `bvr-db` | 3 | SECRET |
| abis-svc | `abis-db` | 3 | SECRET |
| transparency-svc | `audit-db` | 3 | PUBLIC |
| bvi-svc | `bvi-db` | 3 | RESTRICTED |
| portal-svc | `portal-db` | 2 | PUBLIC |

All clusters use:
- PostgreSQL 17 (same major as local testcontainers profile per ADR-0003).
- WAL archiving to MinIO via pgBackRest (30-day PITR window).
- Logical replication slots for Debezium CDC.
- TLS between application and server via cert-manager certificates.

## Install (local overlay)

```bash
# CloudNativePG operator:
helm upgrade --install cnpg cnpg/cloudnative-pg \
  --namespace cnpg-system --create-namespace

# Local single-instance cluster:
kubectl apply -k platform/postgres/local/
```

## Dynamic credentials

Services obtain PostgreSQL credentials from OpenBao's `database/postgres` mount. Credentials rotate every 1 hour. Services use the `eel-resilience-*` connection-pool wrapper to handle rotation without restart.

```bash
# Issue a credential manually (dev/break-glass):
vault read database/postgres/creds/rts-svc-role
```

## Verify replication

```bash
kubectl exec -n eel-db rts-db-1 -- \
  psql -U postgres -c "SELECT * FROM pg_stat_replication;"
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single instance, no WAL archiving, static password |

## Operator responsibilities

- Cluster definitions are managed by ArgoCD. Never alter a `Cluster` resource by hand; edit the overlay and let ArgoCD sync.
- Monitor `cnpg_pg_wal_files_total` and `cnpg_replication_lag` in Grafana.
- PITR restore procedure: `platform/pgbackrest/pitr-restore-procedure.md`.

## Related docs

- `platform/pgbackrest/README.md`: WAL archiving and backup
- `platform/debezium/README.md`: logical replication for CDC
- `platform/openbao/README.md`: dynamic credential provider
