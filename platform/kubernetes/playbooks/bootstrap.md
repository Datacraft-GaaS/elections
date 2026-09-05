# Playbook: Kubernetes Cluster Bootstrap

> **Trigger**: First-time cluster bring-up, or disaster recovery requiring fresh cluster.

## Prerequisites

- All nodes provisioned per `node-pool-manifest.yaml`
- Vault / OpenBao unsealed
- DNS records for `k8s-primary.nairobi.eel.local`, `k8s-mirror.mombasa.eel.local`
- Load balancer for kube-apiserver on port 6443
- Wazuh + Falco + Tetragon agents installable on each node

## Steps

### Step 1: Generate `ClusterConfiguration` from template
```bash
# On first control-plane node
cd platform/kubernetes
# Substitute real node IPs, certificates from Vault, etc.
envsubst < cluster-bootstrap-template.yaml > /tmp/kubeadm-config.yaml
```

### Step 2: Initialize control plane
```bash
sudo kubeadm init --config /tmp/kubeadm-config.yaml --upload-certs
# Save join command output (for adding other control planes)
```

### Step 3: Join additional control plane and worker nodes
```bash
# On each remaining control-plane node
sudo kubeadm join <endpoint> --token <token> --discovery-token-ca-cert-hash <hash> --control-plane --certificate-key <key>
# On each worker node
sudo kubeadm join <endpoint> --token <token> --discovery-token-ca-cert-hash <hash>
```

### Step 4: Apply node taints/labels
```bash
kubectl apply -f node-pool-manifest.yaml
# Verify taints:
kubectl get nodes -o custom-columns=NAME:.metadata.name,TAINTS:.spec.taints
```

### Step 5: Apply namespaces + security policy
```bash
kubectl apply -f namespaces.yaml
kubectl apply -f security-policy.yaml
# Verify:
kubectl get ns | grep ^eel-
kubectl get clusterpolicies
```

### Step 6: Install CNI (Cilium)
```bash
kubectl apply -f ../cilium/values.yaml  # via Helm
# Verify:
kubectl get pods -n kube-system -l k8s-app=cilium
```

### Step 7: Verify audit chain
```bash
# Check kube-apiserver audit log
kubectl exec -n kube-system <kube-apiserver-pod> -- ls /var/log/kubernetes/audit.log
# Verify audit log rotation
kubectl exec -n kube-system <kube-apiserver-pod> -- df -h /var/log/kubernetes
```

## Pass Criteria

- All 14 EEL namespaces created
- All 5 node pools with correct taints
- All 5 Kyverno cluster policies applied
- All 6 control plane + 18 worker nodes (primary) joined
- Cilium CNI installed
- kube-apiserver audit log writing successfully
- Audit events include: `auditId` (UUIDv7) + `proofHash` (SHA-256) + `Ed25519` signature

## References

- `cluster-bootstrap-template.yaml` (cluster config)
- `node-pool-manifest.yaml` (5 node pools)
- `namespaces.yaml` (14 namespaces)
- `security-policy.yaml` (PSS, Kyverno, Cilium)
- ADR 0004 (Platform), ADR 0019 (Edge topology), ADR 0022 (Release/DR)
- M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST
- Elections Act §34 (tamper-evident audit chain)
- NIST SP 800-53 AU-2/AU-10/SC-8/SI-4

## Audit Reference (5-axis)

- Where: cluster bootstrap site (Nairobi DC for primary, Mombasa DC for mirror)
- When: bootstrap start/end timestamps
- Who: bootstrap operator (actorId, actorRole)
- Why: cluster bring-up or DR
- How: kubeadm init/join + Cilium install + ArgoCD bootstrap
- auditId: UUIDv7 from uuid7str()
- proofHash: SHA-256 of bootstrap manifest
- signature: Ed25519 signature from HSM signer
- correlation_key: bootstrap-001
- M3_device_id: cluster-{site}-001
