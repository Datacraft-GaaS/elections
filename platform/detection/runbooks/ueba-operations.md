# Runbook: UEBA Operations

## Health Checks

```bash
# Check baseline-learner and anomaly-detector pods
kubectl get pods -n eel-observability -l app=ueba

# Check baseline table population
psql "$EEL_DB_DSN" -c "SELECT count(*) FROM eel_ueba_baselines;"

# Check for recent alerts
psql "$EEL_DB_DSN" -c "SELECT * FROM eel_ueba_alerts ORDER BY timestamp DESC LIMIT 10;"
```

## Common Issues

| Issue | Diagnosis | Resolution |
|---|---|---|
| No baselines after 14 days | baseline-learner not running | Check Loki query permissions |
| Alerts not reaching ECC | PostgreSQL NOTIFY not working | Check LISTEN connection on ECC dashboard |
| Z-score always 0 | std = 0 (no variance) | Expected for new entities — baseline not yet learned |
| Loki query failing | Network policy blocking | Check Cilium network policy for Loki access |

## Configuration

- Window: 14 days (configurable in ueba-config.yaml)
- Z-score threshold: 3.0 (WARNING), 5.0 (CRITICAL)
- Cooldown: 300 seconds per entity
- Features: hour_of_day, query_rate_per_minute, syscall_entropy, bytes_egressed, distinct_tables_accessed

## References

- `platform/detection/ueba/README.md`
- `platform/detection/ueba/ueba-config.yaml`
- ADR 0009 (Resilience)
