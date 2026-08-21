# platform/kubernetes/

Kubernetes cluster manifests and node configuration for the eel IEMS. This directory is intentionally sparse: most cluster state is managed by higher-level operators (CloudNativePG, Strimzi, Rook-Ceph) whose configs live in their own `platform/` subdirectories.

## Contents

This directory is currently empty. It serves as the conventional location for:

- kubeadm `ClusterConfiguration` YAML (generated at cluster bring-up; not committed because it contains node-specific addresses)
- Node pool definitions (if using a managed K8s service)
- Custom Resource Definition installs not owned by an operator Helm chart

## Cluster topology

| Site | Role | Nodes |
|---|---|---|
| Primary (Nairobi DC) | Active production | 6 control plane + 18 worker nodes |
| Mirror (Mombasa DC) | Active DR standby | 3 control plane + 9 worker nodes |
| Edge sites | Read-only / polling station relay | 1-3 nodes per site |

Node pools:
- `system`: control plane and system pods
- `compute`: general application workloads
- `eel-hsm`: HSM signer pods only (taint `eel.internal/hsm=true:NoSchedule`)
- `storage`: Rook-Ceph OSD nodes (taint `ceph.rook.io/osd=true:NoSchedule`)

## Bring-up

See `docs/runbooks/cluster-bootstrap.md` for the full procedure. Summary:

1. Provision nodes per `docs/infra/node-specs.md`.
2. Run kubeadm init on the first control plane node.
3. Join remaining control plane and worker nodes.
4. Apply the Cilium CNI (`platform/cilium/`).
5. Bootstrap ArgoCD (`platform/argocd/`) to take over all further platform installation.

## Operator responsibilities

- Kubernetes version upgrades follow the `docs/runbooks/k8s-upgrade.md` procedure (one minor version at a time, primary then mirror).
- Do not commit kubeadm configs with real node IPs or certificates; these are generated on-site.
- Node taints and labels are applied by the bootstrap script and enforced by Kyverno.

## Related docs

- `platform/argocd/README.md`: GitOps controller installed after cluster bootstrap
- `platform/cilium/README.md`: CNI applied before any workload
- `docs/runbooks/cluster-bootstrap.md`: first-time cluster setup
