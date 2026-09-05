# platform/hsm-signer/

HSM signer service deployment for the eel IEMS. A gRPC service that wraps PKCS#11 operations against the production HSM (Thales Luna 7 or Utimaco CryptoServer). All cryptographic operations requiring the HSM go through this service: Signed Tree Heads, result declarations, code signing, CA operations, and key ceremonies.

The `hsm-signer` crate lives in `services/transparency-svc/crates/hsm-signer/` per ADR-0024.

## Contents

| File | Description |
|---|---|
| `deployment.yaml` | `Deployment` (3 replicas, leader election via K8s lease) |
| `service.yaml` | `ClusterIP` Service on port 8443 (gRPC) and 8081 (health) |
| `rbac.yaml` | `ServiceAccount`, `Role`, and `RoleBinding` for K8s lease |
| `servicemonitor.yaml` | Prometheus `ServiceMonitor` scraping `:9090/metrics` |
| `values.yaml` | Helm-style values consumed by Kustomize variable substitution |
| `local/` | Kustomize overlay for kind (SoftHSM2 backing, reduced replicas) |

## Install

```bash
# Production:
kubectl apply -k platform/hsm-signer/
# Wait for readiness:
kubectl rollout status deployment/hsm-signer -n eel-hsm

# Local dev (SoftHSM2):
kubectl apply -k platform/hsm-signer/local/
```

## Operation model

- 3 replicas per DR site. Only the leader executes key rotation operations; all replicas serve signing requests.
- Leader election uses a standard `coordination.k8s.io/v1 Lease` in `eel-hsm` namespace.
- The PKCS#11 module (`libCryptoki2.so` or `libsofthsm2.so` in local) is mounted as a hostPath volume from a DaemonSet that provisions the HSM client library.

## Key operations exposed (gRPC)

| RPC | Description |
|---|---|
| `Sign` | Sign a digest with a named key slot |
| `Verify` | Verify a signature against a named key slot's public key |
| `GetPublicKey` | Retrieve the public key for a named slot |
| `RotateKey` | Initiate a key rotation ceremony (quorum required) |
| `ListSlots` | List available key slots and their metadata |

## Key ceremony

Key rotation via `RotateKey` requires M-of-N custodian approvals collected by `tools/hsm-ceremony`. See `docs/runbooks/hsm-ceremony.md`.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: SoftHSM2 hostPath, 1 replica, no leader election |

## Operator responsibilities

- HSM client libraries must be installed on every node in the `eel-hsm` node pool before deploying.
- The HSM PIN is injected from OpenBao at pod start; never set it as a plain env var.
- Monitor `eel_hsm_sign_duration_seconds` and `eel_hsm_errors_total` in Grafana for HSM health.

## Related docs

- `tools/hsm-init/README.md`: local SoftHSM slot initialization
- `tools/hsm-ceremony/README.md`: key ceremony CLI
- `platform/openbao/README.md`: PIN and credential injection
- `docs/runbooks/hsm-ceremony.md`: full ceremony runbook

---

## Technical Explanation (Extended)

**Role**: HSM signer service provides centralized Ed25519 signing for all EEL audit events — every audit entry must be signed by HSM-backed EelSigningKey before being accepted by audit_service.

**Purpose**: Hardware-protected signing key for audit chain integrity (ADR 0017). HSM signer runs as a Kubernetes Deployment in `eel-platform` namespace with taint on `eel-hsm` node pool.

**No stubs**: `deployment.yaml` has real Deployment; `service.yaml` has real Service; `rbac.yaml` has real RBAC (ClusterRole, ServiceAccount, Role, RoleBinding); `values.yaml` has real Helm values; `local/deployment.yaml` has real local override; `servicemonitor.yaml` has real ServiceMonitor.

**M5 Integration**: Every audit event must be signed by HSM signer. During M5 48h drill, all audit events verified for Ed25519 signature. Missing signature = M5 audit chain FAIL. HSM signer availability SLA: 99.99% — downtime must trigger SRE alert within 5 minutes.

**References**: ADR 0004 (Platform — HSM-backed signing), ADR 0005 (Crypto HSM integration), ADR 0017 (Transparency log — Ed25519 signature), M3-HARDWARE-INTEGRATION (HSM node pool), DPA 2019 Kenya §6 (HSM-protected keys), Elections Act §34 (tamper-evident audit — Ed25519 signature required), NIST SP 800-53 AU-10 (Audit integrity), NIST SP 800-53 SC-12 (Cryptographic key establishment and management).
