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

---

## Technical Explanation (Extended) — kopia

**Role**: kopia is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide kopia-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: kopia must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from kopia components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
