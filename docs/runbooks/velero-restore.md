# Velero namespace restore

## Purpose

Restore a Kubernetes namespace from a Velero backup when stateful resources (Deployments, PVCs, ConfigMaps, Secrets) are lost or corrupted. Velero backups are stored in MinIO (S3-compatible object store).

## Prerequisites

- `velero` CLI installed and configured with the correct kubeconfig context.
- MinIO backend is accessible (`velero backup get` returns results).
- The target namespace is identified as `<NAMESPACE>` (e.g., `eel-system`).
- An incident ticket open as `<TICKET>`.
- The namespace to restore to is either deleted or the restore uses `--existing-resource-policy=update`.

## Commands

### Step 1: list available backups

```bash
velero backup get
# find the most recent successful backup for the affected namespace
# note the backup name as <BACKUP_NAME>
```

### Step 2: inspect the backup

```bash
velero backup describe <BACKUP_NAME> --details
# confirm the backup includes the target namespace and its PVCs
```

### Step 3: delete the damaged namespace (if corrupted, not just missing)

```bash
kubectl delete namespace <NAMESPACE>
# wait for deletion:
kubectl wait --for=delete namespace/<NAMESPACE> --timeout=120s
```

### Step 4: restore from backup

```bash
velero restore create <RESTORE_NAME> \
  --from-backup <BACKUP_NAME> \
  --include-namespaces <NAMESPACE> \
  --wait
# monitor restore progress:
velero restore describe <RESTORE_NAME>
velero restore logs <RESTORE_NAME>
```

### Step 5: verify PVC binding

```bash
kubectl get pvc -n <NAMESPACE>
# all PVCs must show STATUS=Bound
# if any PVC is Pending, check StorageClass availability:
kubectl describe pvc <PVC_NAME> -n <NAMESPACE>
```

### Step 6: check pod readiness

```bash
kubectl get pods -n <NAMESPACE>
# all pods should reach Running/Ready within 2 minutes
# investigate any pods stuck in Init or CrashLoopBackOff:
kubectl logs -n <NAMESPACE> <POD_NAME> --tail=50
```

### Step 7: run smoke tests

```bash
for svc in bvr-svc bvi-svc rts-svc abis-svc transparency-svc; do
  echo -n "$svc: "
  kubectl exec -n <NAMESPACE> deploy/$svc -- \
    wget -qO- http://localhost:8080/health 2>/dev/null || echo "UNAVAILABLE"
done
```

### Step 8: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.velero-restore.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"backup_name\":\"<BACKUP_NAME>\",\"namespace\":\"<NAMESPACE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. All PVCs show `STATUS=Bound`.
2. All pods are `Running` and `Ready`.
3. Smoke tests return HTTP 200 for all critical services.
4. Application-level data spot-check: compare record counts against monitoring snapshots.
5. Transparency log contains the velero-restore leaf.

## Rollback procedure

If the restore is incomplete or produces incorrect state, delete the namespace again and restore from an earlier backup. List backups with `velero backup get` and choose a backup predating the incident. Document each attempt in the ticket.

If MinIO is unavailable, escalate to `hsm-loss.md` for object store recovery or use `pgbackrest-pitr.md` for database-level restore.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge)
- Storage Lead for MinIO issues (on-call via PagerDuty rotation)

## Estimated duration

15 to 30 minutes depending on namespace size and PVC data volume.

## Chaos-drill reference

This runbook is exercised by the backup store loss phase of: `chaos/scenarios/eel-minio-site-loss.yaml`

Drill schedule: quarterly, per `docs/runbooks/backup-restore-drill.md`.
