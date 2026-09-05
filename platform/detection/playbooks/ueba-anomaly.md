# Playbook: UEBA Anomaly Investigation

> **Trigger**: UEBA alert raised via PostgreSQL NOTIFY to ECC dashboard (severity WARNING or CRITICAL).

## Steps

### Step 1: Receive alert
```bash
# ECC dashboard receives PostgreSQL NOTIFY on channel eel.ecc.ueba_alert.v1
# Alert payload: entity_id, entity_type, features_breached, z_scores, severity, timestamp
```

### Step 2: Fetch baseline for entity
```bash
# Query eel_ueba_baselines for the entity
psql "$EEL_DB_DSN" -c "SELECT * FROM eel_ueba_baselines WHERE entity_id='$ENTITY_ID';"
```

### Step 3: Identify which feature breached threshold
```bash
# Check z_scores in alert
# WARNING: z > 3.0
# CRITICAL: z > 5.0
# Features: hour_of_day, query_rate_per_minute, syscall_entropy, bytes_egressed, distinct_tables_accessed
```

### Step 4: Correlate with audit chain
```bash
# Query audit_service for events involving this entity
curl -s "https://audit.eel.local/v1/events?entity_id=$ENTITY_ID" \
  -H "Authorization: Bearer $AUDIT_TOKEN" | jq '.'
```

### Step 5: Record investigation in audit chain
```bash
curl -X POST "https://audit.eel.local/v1/events" \
  -H "Authorization: Bearer $AUDIT_TOKEN" \
  -d '{
    "eventType": "ueba_investigation",
    "actorId": "SRE-oncall",
    "actorRole": "sre",
    "whenUtc": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
    "auditId": "'$(uuid7str)'",
    "correlation_key": "ueba-investigation-001",
    "entityId": "'$ENTITY_ID'",
    "severity": "'$SEVERITY'",
    "featuresBreached": ['$FEATURES'],
    "zScores": '$ZSCORES'
  }'
```

## Pass Criteria

- Alert investigated within 30 minutes
- Root cause identified or escalated
- Investigation audit event includes: auditId (UUIDv7) + proofHash + Ed25519 signature
- Correlation with: falco events, wazuh alerts, tetragon process events

## References

- `platform/detection/ueba/README.md`
- `platform/detection/ueba/anomaly-detector.py`
- ADR 0009 (Resilience), ADR 0017 (Transparency log)
- M5-SMS-USB-MANIFEST (M5 48h drill — UEBA must detect anomalies)
