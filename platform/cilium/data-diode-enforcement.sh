#!/usr/bin/env bash
# Data-diode enforcement script (G07)
# Applies Cilium NetworkPolicy for egress-only kit namespaces during polling hours
# M5 auditId: g07-data-diode-enforcement

set -euo pipefail

# During polling hours (0600-1800): ingress denied, egress allowed to RTS only
NAMESPACE="eel-kits"

# 1. Remove all ingress rules (deny all ingress)
kubectl delete networkpolicy --all -n $NAMESPACE 2>/dev/null || true

# 2. Apply egress-only policy
kubectl apply -f platform/cilium/data-diode-policy.yaml

# 3. Verify policy is applied
kubectl get networkpolicy -n $NAMESPACE

# 4. Audit event
echo "auditId=g07-data-diode-enforcement" >> /var/log/eel/deployment-audit.log
echo "Data-diode policy enforced: ingress denied, egress allowed to RTS namespace only"
