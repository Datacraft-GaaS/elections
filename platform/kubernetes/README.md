# platform/kubernetes/

> Cluster foundation for EEL election infrastructure. Kubernetes is the **control plane** that orchestrates all services, enforces security policies, and provides the runtime substrate for the M0→M5 sequential maturity model.

## Role & Purpose

Kubernetes is the **lowest layer of the platform stack** — everything runs on it:
- **Services**: all 20 backend services (abis-svc, bvr-svc, bvi-svc, etc.) run as Deployments/StatefulSets
- **Storage**: CloudNativePG (PostgreSQL), Rook-Ceph (object/block storage), MinIO (S3-compatible)
- **Messaging**: Kafka + Karapace (schema registry), Debezium (CDC), MirrorMaker 2 (DR replication)
- **Identity**: Keycloak (OIDC), Spicedb (Zanzibar authorization), SPIRE (mTLS SVIDs), OpenBao (secrets)
- **Ingress**: Kong (API gateway), Envoy Gateway (HTTP/gRPC ingress)
- **Security**: Kyverno (policy enforcement), Cilium (CNI + network policy), Tetragon (eBPF runtime enforcement), Falco (kernel syscall detection)
- **Observability**: Prometheus + Grafana + Loki + Tempo + OTEL (metrics/logs/traces)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Control Plane (6 nodes primary + 3 nodes DR)          │
│  ┌──────────┐ ┌──────────┐ ┌────────────────────────┐ │
│  │ kube-apiserver (TLS mutual) │ etcd (3 replicas)   │ │
│  │ kube-controller-manager │ kube-scheduler          │ │
│  └──────────────────────────────────────────────────┘ │
│  M5 Audit: every API call → audit.log → audit.eel.local │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────────────────┐
│  Node Pools                                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ system   │ │ compute  │ │ eel-hsm  │ │ storage  │   │
│  │ (6+3)   │ │ (18+9)  │ │ (HSM pods)│ │ (Ceph)  │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│  Edge sites: 1-3 nodes per polling station relay           │
└──────────────────────────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────────────────┐
│  Workloads (namespaces per service)                       │
│  eel-system → eel-abis → eel-biometric → eel-field-mobile│
│  eel-rts → eel-crms → eel-portal → eel-governance        │
│  eel-ai-platform → eel-observability → eel-platform      │
│  eel-edge → eel-dr                                        │
└──────────────────────────────────────────────────────────┘
```

## Files

| File | Purpose |
|---|---|
| `README.md` | This file — cluster overview, topology, bring-up, operator responsibilities |
| `namespaces.yaml` | All 14 EEL namespaces with PSS enforcement, audit annotations, telemetry annotations |
| `cluster-bootstrap-template.yaml` | kubeadm `ClusterConfiguration` template (no secrets/IPs — generated on-site) |
| `node-pool-manifest.yaml` | Node pool definitions: system (6+3), compute (18+9), HSM, storage, edge |
| `security-policy.yaml` | PSS baseline/restricted, Kyverno audit-agent enforcement, Cilium zero-trust, node taints |

## Cluster Topology

### Sites

| Site | Role | Nodes | RPO | RTO | M5 Relevance |
|---|---|---|---|---|---|
| Primary (Nairobi DC) | Active production | 6 control-plane + 18 workers | 0 | 0 | All services active |
| Mirror (Mombasa DC) | Active DR standby | 3 control-plane + 9 workers | 5 min | 15 min | Read-only / failover |
| Edge sites | Polling station relay | 1-3 nodes per site | 24h | 72h | Offline-first, election-day |

### Node Pools

| Pool | Taint | Workloads | MDM Requirement |
|---|---|---|---|
| `system` | `node-role.kubernetes.io/control-plane` | kube-system, Cilium, etcd | M5 MDM (device policy baseline) |
| `compute` | none | General workloads | M5 MDM (device policy baseline) |
| `eel-hsm` | `eel.internal/hsm=true:NoSchedule` | HSM signer pods only | M5 MDM + HSM isolation |
| `storage` | `ceph.rook.io/osd=true:NoSchedule` | Rook-Ceph OSD nodes | M5 MDM + storage isolation |
| `edge` | `eel.internal/edge=true:NoSchedule` | Edge relay, field-mobile | M3 MDM (FLAG_SECURE, kiosk mode) |

## M5 Audit Chain Integration

Every kube-apiserver request generates an **audit event** that flows through the M5 audit chain:

```
kube-apiserver (audit-log /var/log/kubernetes/audit.log)
  → audit_service (eventType: api_call, auditId: UUIDv7, proofHash: SHA-256, Ed25519 sig)
    → wazuh agent (rule 100110-101013)
      → falco (kernel syscall detection)
        → tetragon (eBPF runtime enforcement)
          → audit_shadow (encrypted replay log at /var/lib/eel/audit_shadows)
            → M5 48h drill verification
```

**Audit log configuration**:
- Path: `/var/log/kubernetes/audit.log`
- Retention: 30 days, 10 backups, 100MB max size
- Format: JSON structured (per Kubernetes audit policy)
- All events include `auditId` (UUIDv7) + `proofHash` (SHA-256) + `Ed25519` signature
- Correlation: `correlation_key` links to `detection/canary-tokens/correlation_routing.md`

## Security Posture

### Pod Security Standards (PSS)

| Namespace | PSS Level | Reason |
|---|---|---|
| `eel-system`, `eel-observability`, `eel-platform`, `eel-edge` | **baseline** | System/observability — minimal restrictions |
| All other namespaces | **restricted** | No privileged containers, no host namespaces, no dangerous capabilities |

### Capabilities Blocked (restricted PSS)

```
CAP_SYS_ADMIN    # disallow privilege escalation
CAP_SYS_PTRACE   # disallow process tracing (except tetragon agent)
CAP_SYS_MODULE   # disallow kernel module loading
CAP_MKNOD        # disallow device node creation
CAP_NET_RAW      # disallow raw sockets (except edge network)
```

### Cilium Network Policy (Zero Trust)

- Default deny: all ingress and egress blocked unless explicitly allowed
- Allowed egress: OTEL endpoint (4317/TCP), audit endpoint (8443/TCP, 443/TCP), telemetry
- Allowed ingress: cluster-internal DNS, kube-apiserver
- M5 hardening: all non-allowed traffic triggers Falco ALERT (network_exfil rules)

### Kyverno Policies (Enforced)

| Policy | Action | Scope |
|---|---|---|
| `eel-verify-audit-agent` | **Enforce** | All m2+ pods must have audit-agent + falco + tetragon sidecars |
| `require-otel-annotations` | Enforce | All pods must have `eel/internal/telemetry: true` annotation |
| `forbid-hostpath` | Enforce | No HostPath volumes |
| `forbid-latest-tag` | Enforce | No `latest` image tags |
| `forbid-nodeport` | Enforce | No NodePort services |
| `forbid-privileged` | Enforce | No privileged containers |
| `require-probes-and-limits` | Enforce | All containers need liveness/readiness probes + resource limits |
| `require-signed-images` | Enforce | Images must be signed (Cosign / Notary) |
| `require-svid-label` | Enforce | Pods must have SPIRE SVID label for mTLS |

## M0→M5 Sequential Deployment

Kubernetes is the substrate for all M0→M5 deployment:

| Stage | What | Kubernetes Resource |
|---|---|---|
| M0 | Audit system, observability | `eel-system` namespace, Prometheus, Loki, Tempo |
| M1 | ABIS, CRMS, portal | `eel-abis`, `eel-crms`, `eel-portal` namespaces |
| M2 | BVR, BVI, RTS | `eel-biometric`, `eel-rts` + HSM node pool |
| M3 | Field mobile, edge | `eel-field-mobile`, `eel-edge` + M3 MDM |
| M4 | Governance, AI platform | `eel-governance`, `eel-ai-platform` |
| M5 | DR, M5 hardening | `eel-dr`, full audit chain, canary tokens, tetragon policies |

## Bring-Up Sequence

```
1. Provision nodes per node-pool-manifest.yaml (system, compute, eel-hsm, storage, edge)
   → Apply node taints + labels via bootstrap script (enforced by Kyverno)

2. kubeadm init on first control-plane node
   → Generate ClusterConfiguration from cluster-bootstrap-template.yaml (NO real IPs/certs)

3. Join remaining control-plane and worker nodes
   → All nodes must have wazuh agent + falco agent + tetragon agent registered

4. Install Cilium CNI (platform/cilium/) — before any workload
   → Enforce network policy from security-policy.yaml

5. Install kube-apiserver audit policy
   → Enable audit-log to /var/log/kubernetes/audit.log

6. Apply namespaces.yaml (all 14 EEL namespaces with PSS)

7. Apply security-policy.yaml (Kyverno cluster policies)

8. Bootstrap ArgoCD (platform/argocd/) — GitOps takes over

9. Bootstrap all other platform operators via ArgoCD:
   - cert-manager (TLS certificates)
   - Spicedb (authorization)
   - SPIRE (mTLS SVIDs)
   - OpenBao (secrets)
   - Keycloak (OIDC)
   - Kafka + Karapace + Debezium
   - PostgreSQL (CloudNativePG)
   - Rook-Ceph (storage)
   - Kong + Envoy Gateway (ingress)
   - Observability stack (Prometheus + Grafana + Loki + Tempo + OTEL)
   - Velero (backup)

10. Apply service deployments via ArgoCD ApplicationSets
    (platform/argocd/applicationsets/*)

11. Verify M0→M1→M2→M3→M4→M5 sequential deployment
    → Run M5 48h drill (detection/playbooks/02_correlation_verify.md)
```

## Operator Responsibilities

| Action | Procedure | Frequency |
|---|---|---|
| Kubernetes version upgrade | `docs/runbooks/k8s-upgrade.md` (one minor at a time, primary then mirror) | Per N-2 policy |
| Node join | Verify wazuh + falco + tetragon agents registered before joining | Per event |
| Kyverno policy change | ArgoCD PR + review + test in `eel-edge` namespace | Per change |
| Node taint/label change | Bootstrap script + Kyverno enforcement | Per event |
| Cilium version upgrade | `platform/cilium/` values update + canary rollout | Per release |

## References

- ADR 0004 (Platform): cluster topology, PSS, audit chain
- ADR 0006 (Build & supply chain): image signing, Cosign/Notary verification
- ADR 0009 (Resilience posture): RPO/RTO, backup, DR
- ADR 0010 (Edge & DDoS): Cilium network policy, zero trust
- ADR 0019 (Regional edge topology): edge site design, polling station relay
- ADR 0022 (Release & DR): M5 sequential rules, cluster upgrade procedure
- M3-HARDWARE-INTEGRATION: HSM node pool, device isolation
- M3-SMS-USB-MANIFEST: GSM/USB device handling on edge nodes
- M5-SMS-USB-MANIFEST: M5 48h drill, audit chain verification
- DPA 2019 Kenya §6: audit chain, data protection on cluster
- Elections Act §34: tamper-evident logging, audit integrity
- NIST SP 800-53 AU-2 (Audit events), AU-10 (Audit integrity), SC-8 (Communications security), SI-4 (System monitoring)
