# Playbook: debezium Operations

> **Trigger**: Standard operational procedure for debezium.

## Prerequisites

- Kubernetes cluster healthy
- debezium installed via ArgoCD
- Namespace: eel-platform or eel-system

## Steps

### Step 1: Verify debezium is healthy
```bash
kubectl get pods -n eel-platform -l app=debezium
```

### Step 2: Check logs
```bash
kubectl logs -n eel-platform -l app=debezium --tail=100
```

### Step 3: Verify configuration
```bash
kubectl get configmap -n eel-platform debezium-config
kubectl describe configmap -n eel-platform debezium-config
```

### Step 4: Verify audit chain integration
```bash
# Check audit events include auditId + proofHash + Ed25519 signature
kubectl logs -n eel-platform -l app=debezium | grep auditId | jq '.'
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
- Wazuh: No debezium-specific alerts
- Falco: No debezium-specific ALERTs

## References

- platform/debezium/README.md
- ADR 0004 (Platform), ADR 0009 (Resilience), ADR 0010 (Edge/DDoS), ADR 0019 (Edge topology), ADR 0022 (Release/DR)
- M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST
- DPA 2019 Kenya §6, Elections Act §34
- NIST SP 800-53 AU-2, AU-10, SI-4, SC-8
