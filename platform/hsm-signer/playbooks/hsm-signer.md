# Playbook: HSM Signer Key Rotation

> **Trigger**: Scheduled key rotation (per key rotation policy), or emergency key rotation after suspected compromise.

## Prerequisites

- OpenBao unsealed and accessible
- HSM signer pod healthy
- Audit chain intact (all services can reach HSM signer)
- SRE on-call notified

## Steps

### Step 1: Verify HSM signer health
```bash
kubectl get pods -n eel-platform -l app=hsm-signer
kubectl logs -n eel-platform -l app=hsm-signer --tail=20
```

### Step 2: Check current key metadata
```bash
kubectl exec -n eel-platform deploy/hsm-signer -- hsm-signer status
# Verify key version, creation time, rotation schedule
```

### Step 3: Generate new key version in OpenBao
```bash
# Request new key version from OpenBao
vault write -f openbao/transit/keys/eel-signing-key/rotate
# Verify:
vault read openbao/transit/keys/eel-signing-key
```

### Step 4: Update HSM signer to use new key version
```bash
# Update configmap
kubectl patch configmap hsm-signer-config -n eel-platform \
  --type merge -p '{"data":{"signing_key_version":"new"}}'
# Restart HSM signer
kubectl rollout restart deployment/hsm-signer -n eel-platform
kubectl rollout status deployment/hsm-signer -n eel-platform
```

### Step 5: Verify new key in use
```bash
kubectl exec -n eel-platform deploy/hsm-signer -- hsm-signer status
# Verify new key version active
```

### Step 6: Record key rotation in audit chain
```bash
curl -X POST "https://audit.eel.local/v1/events" \
  -H "Authorization: Bearer $AUDIT_TOKEN" \
  -d '{
    "eventType": "hsm_key_rotation",
    "actorId": "SRE-operator",
    "actorRole": "sre",
    "whenUtc": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
    "auditId": "'$(uuid7str)'",
    "correlation_key": "hsm-rotation-001",
    "keyId": "eel-signing-key-v2"
  }'
```

## Pass Criteria

- HSM signer: Running, responding to signing requests
- All audit events signed with new key version
- Key rotation audit event includes: auditId (UUIDv7) + proofHash + Ed25519 signature
- No service disruption (new key version backwards-compatible for verification)

## SRE Escalation

If HSM signer unavailable for > 5 minutes:
1. Page SRE (PagerDuty P1)
2. Audit service falls back to software key (emergency only — must be documented)
3. Emergency key rotation procedure per `docs/runbooks/hsm-emergency.md`

## References

- `platform/hsm-signer/values.yaml`
- ADR 0005 (Crypto HSM), ADR 0017 (Transparency log)
- M3-HARDWARE-INTEGRATION (HSM node pool)
- DPA 2019 Kenya §6 (HSM-protected keys)
