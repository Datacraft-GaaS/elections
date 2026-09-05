# Playbook: ArgoCD GitOps Bootstrap

> **Trigger**: Bootstrap ArgoCD on fresh cluster, or re-bootstrap after ArgoCD failure.

## Prerequisites

- Kubernetes cluster running (see `platform/kubernetes/playbooks/bootstrap.md`)
- `argocd` namespace must exist
- `install.yaml` applied

## Steps

### Step 1: Apply ArgoCD install
```bash
kubectl apply -n argocd -f platform/argocd/install.yaml
# Verify:
kubectl get pods -n argocd -l app.kubernetes.io/name=argocd-server
```

### Step 2: Patch SSO config (Keycloak OIDC)
```bash
kubectl apply -n argocd -f platform/argocd/install.yaml
# Verify RBAC:
kubectl get configmap -n argocd argocd-rbac-cm
```

### Step 3: Apply AppProjects
```bash
kubectl apply -n argocd -f platform/argocd/app-projects.yaml
kubectl apply -n argocd -f platform/argocd/projects/
# Verify:
kubectl get appprojects -n argocd
```

### Step 4: Apply ApplicationSets
```bash
kubectl apply -n argocd -f platform/argocd/applicationsets/
kubectl apply -n argocd -f platform/argocd/applicationsets/generated/
# Verify:
kubectl get applicationsets -n argocd
```

### Step 5: Verify sync
```bash
argocd login --sso https://argocd.eel.internal
argocd app list
# All apps should be Healthy/Synced
```

### Step 6: Verify notifications
```bash
# Test Slack notification:
argocd app sync bvr-svc --force
# Check #eel-ops Slack channel
```

## Pass Criteria

- All AppProjects created (`trust-core`, `election-critical`, `operations`, `governance`)
- All ApplicationSets present
- All applications Healthy/Synced
- Notifications routing to Slack `#eel-ops` and PagerDuty P1 for election-critical
- Audit events include `auditId` + `proofHash` + `Ed25519` signature

## References

- `install.yaml` (HA install with OIDC SSO)
- `app-projects.yaml` (AppProject definitions)
- `applicationsets/` (service ApplicationSets)
- `rollouts/` (progressive delivery)
- ADR 0004, 0017, 0022, 0019
