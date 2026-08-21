# platform/kopia/

Kopia backup configuration for non-database stateful state in the eel IEMS. Covers Kubernetes secrets snapshots, configuration files, and any file-system state not managed by pgBackRest or Velero.

## Contents

| File | Description |
|---|---|
| `kopia-repository.yaml` | Kopia repository configuration (S3-backed, AES-256 encryption) |
| `kopia-backup-cronjob.yaml` | Kubernetes `CronJob` for scheduled backups |

## Backup targets

| Target | Path | Schedule | Retention |
|---|---|---|---|
| Keycloak realm exports | `/keycloak-exports/` | Daily 03:00 UTC | 90 days |
| SpiceDB relation backup | `/spicedb-backup/` | Daily 03:00 UTC | 90 days |
| OpenBao unseal keys (encrypted envelope) | `/openbao-keys/` | Weekly Sunday 04:00 UTC | 1 year |
| SPIRE CA bundle | `/spire-bundle/` | Daily 03:00 UTC | 90 days |

## Repository

Backups are stored in `eel-kopia-backups` bucket on MinIO (primary) with a DR copy on the mirror MinIO.

```bash
# Initialize repository (first time):
kopia repository create s3 \
  --bucket eel-kopia-backups \
  --endpoint minio.eel-storage:9000 \
  --access-key "$MINIO_ACCESS_KEY" \
  --secret-access-key "$MINIO_SECRET_KEY" \
  --password "$KOPIA_REPO_PASSWORD"

# Connect to an existing repository:
kopia repository connect s3 \
  --bucket eel-kopia-backups \
  --endpoint minio.eel-storage:9000 \
  --access-key "$MINIO_ACCESS_KEY" \
  --secret-access-key "$MINIO_SECRET_KEY" \
  --password "$KOPIA_REPO_PASSWORD"
```

## Apply CronJob

```bash
kubectl apply -f platform/kopia/kopia-repository.yaml
kubectl apply -f platform/kopia/kopia-backup-cronjob.yaml
```

## Restore

```bash
# List snapshots:
kopia snapshot list

# Restore a specific snapshot:
kopia restore <snapshot-id> /restore-target/
```

## Operator responsibilities

- The repository password is stored in OpenBao (`kv/eel/kopia/repo-password`). Rotate annually.
- Kopia backups are deduplicated and encrypted client-side before upload. The MinIO bucket holds only ciphertext.
- Verify restore integrity quarterly by restoring a snapshot to a scratch namespace.

## Related docs

- `platform/minio/README.md`: backup storage bucket
- `platform/velero/README.md`: Kubernetes object state backup (complementary)
- `platform/pgbackrest/README.md`: PostgreSQL-specific backup
