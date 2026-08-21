# ADR 0004: Platform

Status: accepted
Date: 2026-08-21

## Context

Kenya-sovereign three-site on-prem deployment (Anniversary Towers primary, Bomas mirror sync-replicated, escrow async). GitOps-only per `sw/expect.md`. Immutable images with cosign admission per `sw/ARCHITECTURE.md`. SPIFFE/SPIRE identity per spec.

## Decision

- **Kubernetes 1.31** provisioned with kubeadm; one cluster per site (three total).
- **Container runtime**: containerd.
- **Node OS**: **Ubuntu 24.04 LTS** — ops-team familiarity over Talos's image-based elegance for a 3-year contract. Immutability provided by kernel-signed boot + hardened base image + read-only rootfs where feasible.
- **CNI + service mesh**: **Cilium** (Apache 2.0) providing CNI, service mesh, NetworkPolicy enforcement, and Hubble observability in one component. Linkerd rejected because Cilium consolidates four separate concerns.
- **Workload identity**: **SPIFFE/SPIRE** with HSM-sealed root (see ADR-0005). Every workload gets an SVID (X.509 or JWT) attested from K8s SA + TPM + HSM selectors.
- **Ingress**: **Envoy Gateway** (Apache 2.0).
- **GitOps**: **ArgoCD** with ApplicationSets for per-module fan-out.
- **Container registry**: **Harbor** (Apache 2.0) with cosign verification enforced by Kyverno admission.
- **Admission & policy**: **Kyverno** + **OPA Gatekeeper**. Kyverno rejects unsigned images, enforces mesh membership, blocks risky manifests; OPA carries reusable Rego bundles for domain policy.
- **Secrets injection**: OpenBao Agent Injector (see ADR-0007).
- **Certificate lifecycle**: **cert-manager** with **step-ca** issuers rooted in HSM.
- **Observability**: OpenTelemetry Collector → Prometheus + Loki + Tempo + Grafana OSS. **Vector** (MPL) as node agent. **Mimir** for long-retention metrics.
- **Detection**: **Falco + Tetragon** (eBPF) on every node, **Wazuh** as SIEM with Sigma rules.
- **Storage**: **Rook-Ceph** for block/object/file; MinIO S3 layered on Ceph.
- **Backup**: **Velero** + **Kopia**, pgBackRest for Postgres WAL to MinIO, Velero for cluster state to escrow site.

## Consequences

Positive: single mesh + CNI + policy + observability layer (Cilium/Hubble) rather than four; Ubuntu keeps ops team on familiar ground; every component OSS with a mirrorable release channel; matches the spec's SPIFFE + cosign + GitOps commitments.

Negative: Cilium's newer service-mesh (as of 1.14+) is younger than Linkerd; mitigated by conservative version pinning + rehearsed upgrades. Ubuntu is less minimal than Talos; we lose some CVE-surface reduction in exchange for team familiarity.

## Alternatives rejected

- Talos Linux: technically superior for immutability; team familiarity cost too high for a 3-year contract.
- Linkerd 2 mesh: spec-suggested; Cilium consolidates more concerns without loss of function.
- Istio: heavier, more moving parts, harder to defend under simplicity constraint.
- K3s: too much magic for a system this critical.
- Rancher stack: adds a management plane we do not need.

## Related

ADR-0003 (data infra runs on this), ADR-0005 (HSM-rooted PKI feeds SPIRE + cert-manager), ADR-0006 (Nix + cosign feeds Harbor + Kyverno), ADR-0010 (edge sits in front).
