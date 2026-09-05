# Runbook: Kubernetes Cluster Operations

## Cluster Upgrade

**Reference**: `docs/runbooks/k8s-upgrade.md`

**Trigger**: Kubernetes version upgrade (one minor at a time, primary then mirror).

**Steps**:
1. Drain all nodes in primary site (one at a time).
2. Upgrade kubeadm on all nodes: `apt-get install kubeadm=1.29.x`.
3. Run `sudo kubeadm upgrade apply v1.29.x` on control plane.
4. Upgrade kubelet on control plane nodes.
5. Uncordon control plane nodes.
6. Repeat for worker nodes (drain → upgrade → uncordon).
7. Verify all pods healthy: `kubectl get pods -A`.
8. Repeat for mirror site.
9. Verify ArgoCD sync: `argocd app list`.

**Failure recovery**: If upgrade fails, `kubeadm upgrade apply --force` reverts to previous version.

**Rollback**: `sudo kubeadm upgrade apply v<previous-version>` — takes ~15 minutes.

## Node Replacement

**Trigger**: Node failure or hardware replacement.

**Steps**:
1. Cordon node: `kubectl cordon <node>`.
2. Drain node: `kubectl drain <node> --ignore-daemonsets --delete-emptydir-data`.
3. Remove node from cluster: `kubectl delete node <node>`.
4. Provision replacement node per `node-pool-manifest.yaml`.
5. Join to cluster with `kubeadm join`.
6. Verify wazuh + falco + tetragon agents registered.
7. Uncordon: `kubectl uncordon <node>`.

## Cluster Backup

**Reference**: `platform/velero/` for Velero backup of etcd, Kubernetes resources.

**Trigger**: Scheduled (daily) or before major change.

## Audit Chain Verification

**Trigger**: Verify audit events include `auditId` + `proofHash` + `Ed25519` signature.

**Command**:
```bash
kubectl logs -n kube-system <kube-apiserver-pod> --tail=100 | grep auditId | jq '.'
```

**Pass criteria**: Every API call has auditId (UUIDv7), proofHash (64 hex chars), Ed25519 signature (128 hex chars).

## Common Issues

| Issue | Diagnosis | Resolution |
|---|---|---|
| Node stuck in NotReady | `kubectl describe node` | Check kubelet logs, CNI status |
| Pod stuck in CrashLoopBackOff | `kubectl logs` | Check OOM, missing env vars |
| API server slow | Check etcd latency | Scale etcd, add resources |
| Audit log not writing | Check `/var/log/kubernetes/` | Restart kube-apiserver pod |

## References

- `platform/kubernetes/README.md` — cluster topology
- `platform/cilium/README.md` — CNI
- `platform/argocd/README.md` — GitOps
- `platform/velero/README.md` — backup
- ADR 0004, 0019, 0022
- M5-SMS-USB-MANIFEST (M5 48h drill)
