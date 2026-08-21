# platform/

GitOps-managed platform layer for the eel IEMS, per ADR-0019. All manifests are applied by ArgoCD from this directory tree. No `kubectl apply` by hand in production.

## Tier ordering (bring-up sequence)

| Tier | Components | Rationale |
|---|---|---|
| 0: Cluster | `kubernetes/` | Node provisioning, kubeadm configs, CNI placeholder |
| 1: Network and policy | `cilium/`, `kyverno-policies/`, `envoy-gateway/` | CNI, admission control, and ingress before any workload |
| 2: Storage | `rook-ceph/`, `minio/` | Block/object storage for all stateful services |
| 3: Secrets and PKI | `openbao/`, `cert-manager/`, `spire/` | Vault seal, certificate issuers, workload identity |
| 4: Identity and authz | `keycloak/`, `spicedb/`, `opa-bundles/` | Authentication, fine-grained authz, policy bundles |
| 5: Messaging | `kafka/`, `mirrormaker2/`, `debezium/`, `karapace/` | Event bus and schema registry before application services |
| 6: Data | `postgres/`, `pgbackrest/`, `valkey/`, `opensearch/`, `kopia/`, `velero/` | Databases and backup |
| 7: Registry and GitOps | `harbor/`, `argocd/` | Image registry and GitOps controller |
| 8: Crypto infrastructure | `hsm-signer/` | Signing service backed by HSM |
| 9: Observability | `observability/` | Metrics, logs, traces, SLOs |
| 10: Operations | `kong/`, `election-day-mode/` | API gateway and operational flags |

## Component index

| Directory | Component | Namespace |
|---|---|---|
| `argocd/` | ArgoCD GitOps controller | `argocd` |
| `cert-manager/` | cert-manager PKI automation | `cert-manager` |
| `cilium/` | Cilium CNI and mesh | `kube-system` |
| `debezium/` | Debezium CDC connectors | `eel-kafka` |
| `election-day-mode/` | Election-day freeze flag and policy | `argocd` |
| `envoy-gateway/` | Envoy Gateway ingress/egress | `envoy-gateway-system` |
| `harbor/` | Harbor container registry | `harbor` |
| `hsm-signer/` | HSM signing service | `eel-hsm` |
| `kafka/` | Kafka KRaft cluster and topics | `eel-kafka` |
| `karapace/` | Schema registry | `eel-kafka` |
| `keycloak/` | Keycloak IAM | `eel-iam` |
| `kong/` | Kong API gateway | `eel-gateway` |
| `kopia/` | Kopia backup for non-PG state | `eel-ops` |
| `kubernetes/` | Cluster manifests | `kube-system` |
| `kyverno-policies/` | Admission policies | `kyverno` |
| `minio/` | MinIO object storage | `eel-storage` |
| `mirrormaker2/` | Kafka MirrorMaker2 (DR replication) | `eel-kafka` |
| `observability/` | OTel, Prometheus, Loki, Tempo, Grafana | `eel-obs` |
| `opa-bundles/` | OPA Rego policy bundles | `eel-policy` |
| `openbao/` | OpenBao secrets management | `eel-vault` |
| `opensearch/` | OpenSearch audit log store | `eel-search` |
| `pgbackrest/` | PostgreSQL backup and PITR | `eel-db` |
| `postgres/` | PostgreSQL cluster | `eel-db` |
| `rook-ceph/` | Rook-Ceph distributed storage | `rook-ceph` |
| `spicedb/` | SpiceDB fine-grained authz | `eel-iam` |
| `spire/` | SPIRE workload identity | `spire` |
| `valkey/` | Valkey (Redis-compatible) cache | `eel-cache` |
| `velero/` | Cluster-state backup | `velero` |

## Overlays

Each component that deploys to multiple environments has a `local/` subdirectory with Kustomize overlays for local dev (kind cluster). Production overlays live in `services/<svc>/deploy/overlays/primary` and `mirror`.

## Related docs

- `docs/adr/0019-platform-stack.md`: ADR-0019, platform technology decisions
- `docs/runbooks/cluster-bootstrap.md`: first-time cluster bring-up
- `tools/appset-gen/README.md`: ApplicationSet generator
- `tools/iam-seed/README.md`: IAM bootstrap tool
