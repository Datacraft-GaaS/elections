# Playbook: PostgreSQL Disaster Recovery

> **Trigger**: Primary PostgreSQL failure, or scheduled DR drill.

## Prerequisites

- pgBackRest repository accessible
- CloudNativePG operator healthy
- Barman or pgBackRest at DR site accessible
- WAL archiving to DR site working

## Steps

### Step 1: Detect primary failure
```bash
kubectl get clusters -n eel-postgres
# If cluster status != Healthy → trigger DR
```

### Step 2: Promote DR replica
```bash
# On DR site
kubectl patch cluster eel-postgres-dr --type merge \
  -p '{"spec":{"promotionTrigger":"true"}}'
# Monitor:
kubectl logs -n eel-postgres deploy/eel-postgres-dr --tail=20
```

### Step 3: Update connection strings
```bash
# Update Keycloak database connection
kubectl patch configmap keycloak-db-config -n eel-platform \
  --type merge -p '{"data":{"db_host":"eel-postgres-dr.eel-postgres.svc"}}'
kubectl rollout restart deployment/keycloak -n eel-platform
```

### Step 4: Verify audit chain
```bash
# Check audit_service can write to new primary
curl -s "https://audit.eel.local/v1/health" | jq '.'
```

### Step 5: Record DR failover in audit chain
```bash
curl -X POST "https://audit.eel.local/v1/events" \
  -H "Authorization: Bearer $AUDIT_TOKEN" \
  -d '{
    "eventType": "dr_failover_postgres",
    "actorId": "SRE-oncall",
    "actorRole": "sre",
    "whenUtc": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
    "auditId": "'$(uuid7str)'",
    "correlation_key": "dr-failover-001",
    "previous_primary": "eel-postgres-primary",
    "new_primary": "eel-postgres-dr",
    "RPO": "5min",
    "RTO": "15min"
  }'
```

## RPO/RTO Targets

| Service | RPO | RTO |
|---|---|---|
| audit_service | 5 min | 15 min |
| abis-svc | 5 min | 30 min |
| bvr-svc | 24h (offline-first) | 72h |
| field-mobile-svc | 24h (outbox) | 72h |

## Pass Criteria

- DR replica promoted successfully
- All services connected to new primary
- Audit events include: auditId (UUIDv7) + proofHash + Ed25519 signature
- No data loss (RPO ≤ 5 min verified via WAL sequence comparison)
- DR failover audit event recorded in audit chain

## References

- `platform/postgres/` (CloudNativePG configuration)
- `platform/pgbackrest/` (backup configuration)
- `platform/velero/` (Kubernetes resource backup)
- ADR 0009 (Resilience), ADR 0022 (Release/DR)
- M5-SMS-USB-MANIFEST (M5 DR drill)
