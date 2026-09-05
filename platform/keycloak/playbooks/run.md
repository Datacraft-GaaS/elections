# Playbook: keycloak Operations

> **Trigger**: Standard operational procedure for keycloak.

## Prerequisites

- Kubernetes cluster healthy
- keycloak installed via ArgoCD
- Namespace: eel-platform or eel-system

## Steps

### Step 1: Verify keycloak is healthy
```bash
kubectl get pods -n eel-platform -l app=keycloak
```

### Step 2: Check logs
```bash
kubectl logs -n eel-platform -l app=keycloak --tail=100
```

### Step 3: Verify configuration
```bash
kubectl get configmap -n eel-platform keycloak-config
kubectl describe configmap -n eel-platform keycloak-config
```

### Step 4: Verify audit chain integration
```bash
# Check audit events include auditId + proofHash + Ed25519 signature
kubectl logs -n eel-platform -l app=keycloak | grep auditId | jq '.'
```

### Step 5: Verify telemetry (OTEL)
```bash
# Check telemetry flowing to otel.eel.local:4317
kubectl get endpoints -n eel-observability otel-collector
```

## Pass Criteria

- Pods: Running, all replicas healthy
- Logs: No ERROR or CRITICAL
- Audit: auditId (UUIDv7), proofHash (SHA-256), Ed25519 signature present
- Telemetry: Metrics flowing to Prometheus, traces to Tempo, logs to Loki
- Wazuh: No keycloak-specific alerts
- Falco: No keycloak-specific ALERTs

## References

- platform/keycloak/README.md
- ADR 0004 (Platform), ADR 0009 (Resilience), ADR 0010 (Edge/DDoS), ADR 0019 (Edge topology), ADR 0022 (Release/DR)
- M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST
- DPA 2019 Kenya §6, Elections Act §34
- NIST SP 800-53 AU-2, AU-10, SI-4, SC-8
