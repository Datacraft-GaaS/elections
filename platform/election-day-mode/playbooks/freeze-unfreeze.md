# Playbook: Election-Day Freeze Activation/Deactivation

> **Trigger**: Activate or deactivate election-day freeze mode.

## Activate Freeze

### Step 1: Verify M5 audit chain is intact
```bash
kubectl logs -n eel-observability -l app=audit-service --tail=10 | grep auditId | jq '.'
# Verify auditId (UUIDv7) + proofHash + Ed25519 signature present
```

### Step 2: Activate freeze via ConfigMap
```bash
# Set freeze_active=true and freeze_start/end timestamps
kubectl patch configmap election-day-freeze -n eel-platform \
  --type merge -p '{"data":{"freeze_active":"true","freeze_start":"2024-08-09T06:00:00Z","freeze_end":"2024-08-09T18:00:00Z"}}'
```

### Step 3: Verify Kyverno policy enforces freeze
```bash
# Try to create a pod in eel-biometric namespace during freeze
kubectl run test-pod --image=nginx -n eel-biometric
# Expected: denied by Kyverno freeze policy
```

### Step 4: Verify ArgoCD auto-sync disabled for election-critical
```bash
argocd app get bvr-svc | grep SyncPolicy
# Expected: auto-sync disabled during freeze
```

## Deactivate Freeze

### Step 1: Verify freeze_end time passed
```bash
date -u # Verify current UTC time > freeze_end
```

### Step 2: Deactivate freeze
```bash
kubectl patch configmap election-day-freeze -n eel-platform \
  --type merge -p '{"data":{"freeze_active":"false"}}'
```

### Step 3: Verify ArgoCD auto-sync re-enabled
```bash
argocd app sync bvr-svc --force
argocd app get bvr-svc | grep SyncPolicy
```

### Step 4: Record deactivation in audit chain
```bash
# Write audit event for freeze deactivation
curl -X POST "https://audit.eel.local/v1/events" \
  -H "Authorization: Bearer $AUDIT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "eventType": "freeze_deactivated",
    "actorId": "operator-001",
    "actorRole": "M5_auditor",
    "whenUtc": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
    "auditId": "'$(uuid7str)'",
    "correlation_key": "freeze-deactivate-001"
  }'
```

## Pass Criteria

- Freeze active: ArgoCD auto-sync blocked for election-critical namespaces
- Kyverno denies pod creation/deletion in frozen namespaces
- Audit events include: auditId (UUIDv7) + proofHash + Ed25519 signature
- All systems reference: correlation_key, M5_device_id, actorId

## References

- `platform/election-day-mode/toggle-runbook.md` (full procedure)
- `platform/election-day-mode/freeze-window-policy.yaml`
- ADR 0022 (Release/DR), ADR 0017 (Transparency log)
- M5-SMS-USB-MANIFEST (M5 freeze verification)
- Elections Act §34 (election-day freeze)
