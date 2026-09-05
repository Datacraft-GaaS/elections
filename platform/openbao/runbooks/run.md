# Runbook: $folder Troubleshooting

## Common Issues

| Issue | Diagnosis | Resolution |
|---|---|---|
| Pod stuck in CrashLoopBackOff | \`kubectl logs\` \`kubectl describe pod\` | Check missing configmap/secrets, OOM, dependency |
| Pod stuck in Pending | \`kubectl describe pod\` | Check resources, PVC binding, node selector |
| High memory/CPU | \`kubectl top pod\` | Increase resource limits via ArgoCD |
| Audit events missing | Check audit-service endpoint | Restart pod, check network policy |
| TLS cert expired | \`kubectl get cert\` | cert-manager auto-renews; check if renewal failed |

## Health Checks

\`\`\`bash
# 1. Pod health
kubectl get pods -n eel-platform -l app=$folder -o wide

# 2. Endpoint health
kubectl get endpoints -n eel-platform $folder

# 3. Logs
kubectl logs -n eel-platform -l app=$folder --tail=50 --timestamps

# 4. Audit events
kubectl logs -n eel-platform -l app=$folder | grep auditId | tail -5 | jq '.'

# 5. Wazuh alerts
grep "$folder" /var/ossec/logs/alerts/alerts.json | tail -5 | jq '.'
\`\`\`

## Recovery Procedures

**Pod restart** (non-disruptive):
\`\`\`bash
kubectl rollout restart deployment/$folder -n eel-platform
kubectl rollout status deployment/$folder -n eel-platform
\`\`\`

**Config update**:
\`\`\`bash
kubectl edit configmap $folder-config -n eel-platform
# ArgoCD will auto-sync changes
\`\`\`

## Escalation

- **Level 1**: Check logs, restart pod
- **Level 2**: Check Wazuh/Falco alerts, check audit chain
- **Level 3**: Page SRE (PagerDuty P1 for election-critical services)
- **Level 4**: Failover to DR site per DR plan

## References

- platform/$folder/README.md
- ADR 0004, 0009, 0010, 0019, 0022
- M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST
- DPA 2019 Kenya §6, Elections Act §34
