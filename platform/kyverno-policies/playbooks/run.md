# Playbook: kyverno-policies Operations

> **Trigger**: Standard operational procedure for kyverno-policies.

## Prerequisites

- Kubernetes cluster healthy
- kyverno-policies installed via ArgoCD
- Namespace: eel-platform or eel-system

## Steps

### Step 1: Verify kyverno-policies is healthy
```bash
kubectl get pods -n eel-platform -l app=kyverno-policies
```

### Step 2: Check logs
```bash
kubectl logs -n eel-platform -l app=kyverno-policies --tail=100
```

### Step 3: Verify configuration
```bash
kubectl get configmap -n eel-platform kyverno-policies-config
kubectl describe configmap -n eel-platform kyverno-policies-config
```

### Step 4: Verify audit chain integration
```bash
# Check audit events include auditId + proofHash + Ed25519 signature
kubectl logs -n eel-platform -l app=kyverno-policies | grep auditId | jq '.'
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
- Wazuh: No kyverno-policies-specific alerts
- Falco: No kyverno-policies-specific ALERTs

## References

- platform/kyverno-policies/README.md
- ADR 0004 (Platform), ADR 0009 (Resilience), ADR 0010 (Edge/DDoS), ADR 0019 (Edge topology), ADR 0022 (Release/DR)
- M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST
- DPA 2019 Kenya §6, Elections Act §34
- NIST SP 800-53 AU-2, AU-10, SI-4, SC-8
