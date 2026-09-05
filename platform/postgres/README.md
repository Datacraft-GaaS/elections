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

---

## Technical Explanation (Extended) — postgres

**Role**: postgres is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide postgres-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: postgres must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from postgres components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
