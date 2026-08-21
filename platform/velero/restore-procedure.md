# Velero Cluster-State Restore Procedure

## Prerequisites

- Velero CLI >= 1.13 installed
- `KUBECONFIG` pointing to the target (restored) cluster
- S3 credentials configured: `velero backup-location get`

## Steps

1. List available backups:
   ```
   velero backup get
   ```

2. Inspect a specific backup:
   ```
   velero backup describe daily-cluster-state-<timestamp> --details
   ```

3. Restore to a new namespace prefix (non-destructive first pass):
   ```
   velero restore create --from-backup daily-cluster-state-<timestamp> \
     --namespace-mappings "eel-prod:eel-restore" \
     --include-namespaces eel-prod
   ```

4. Verify restore:
   ```
   velero restore describe <restore-name> --details
   kubectl -n eel-restore get pods
   ```

5. Promote restore (cut over):
   ```
   # Drain eel-prod namespace traffic first (update DNS / ingress weights)
   # Then:
   velero restore create --from-backup daily-cluster-state-<timestamp> \
     --include-namespaces eel-prod \
     --existing-resource-policy=update
   ```

6. Post-restore checks:
   - `kubectl -n eel-prod get pods` -- all Running
   - `kubectl -n eel-prod get pvc` -- all Bound
   - Run smoke test: `tools/dr-failover/target/release/eel-dr-failover report --check-svc rts-svc`

7. Record RTO in DR drill log.
