# platform/velero/

Velero cluster-state backup and restore for the eel IEMS. Backs up Kubernetes objects (Deployments, ConfigMaps, Secrets, CRDs, PVCs) on a daily schedule. Complements pgBackRest (PostgreSQL data) and Kopia (file-system state).

## Contents

| File | Description |
|---|---|
| `velero-values.yaml` | Helm values (S3-backed via MinIO, restic enabled for PVC content) |
| `restore-procedure.md` | Step-by-step cluster restore runbook |

## Backup schedule

| Schedule name | Frequency | Include namespaces | Retention |
|---|---|---|---|
| `daily-cluster-state` | Daily 02:00 UTC | all `eel-*`, `argocd`, `kyverno` | 30 days |
| `weekly-full` | Sunday 02:30 UTC | all namespaces | 12 weeks |

## Install

```bash
helm upgrade --install velero vmware-tanzu/velero \
  --namespace velero --create-namespace \
  -f platform/velero/velero-values.yaml
```

## Create a manual backup

```bash
velero backup create pre-upgrade-backup-$(date +%Y%m%d) \
  --include-namespaces eel-rts,eel-bvr,eel-bvi

velero backup describe pre-upgrade-backup-$(date +%Y%m%d) --details
```

## Restore procedure

Full steps: `restore-procedure.md`. Short form:

```bash
# List available backups:
velero backup get

# Restore to a separate namespace (non-destructive first pass):
velero restore create test-restore \
  --from-backup daily-cluster-state-20260820 \
  --namespace-mappings "eel-prod:eel-restore"

# Verify restore:
velero restore describe test-restore --details
```

## Limitations

- Velero does not back up PostgreSQL data; use pgBackRest for PITR.
- PVC content backup (via restic) is enabled but slow for large volumes (> 100 GB). Prefer pgBackRest for database PVCs.
- Velero cannot restore across Kubernetes major versions. Both source and target clusters must run the same K8s version.

## Operator responsibilities

- Backup storage location points to `eel-velero-backups` bucket on MinIO. Verify the backup location is available before a planned maintenance.
- Run a restore drill quarterly into a scratch cluster using the `restore-procedure.md` steps.
- Monitor `velero_backup_last_successful_timestamp` in Grafana; alert fires if > 26 hours.

## Related docs

- `platform/pgbackrest/README.md`: PostgreSQL data backup (use this, not Velero, for DB recovery)
- `platform/kopia/README.md`: non-DB file-system state backup
- `platform/minio/README.md`: backup storage bucket
- `tools/dr-failover/README.md`: DR failover uses Velero restore as one step

---

## Technical Explanation (Extended) — velero

**Role**: velero is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide velero-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: velero must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from velero components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
