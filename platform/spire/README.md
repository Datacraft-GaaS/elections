# platform/spire/

SPIRE (SPIFFE Runtime Environment) configuration for the eel IEMS. Issues SPIFFE SVIDs to all 18 services and platform components, enabling mTLS without manually managed certificates.

## Contents

| Path | Description |
|---|---|
| `values.yaml` | Helm values for SPIRE server and agent |
| `registration-entries.yaml` | Aggregated registration entry list (generated from `entries/`) |
| `entries/` | One YAML per service or platform component |
| `local/` | Kustomize overlay for kind (disk-based attestor, no TPM) |

## Install

```bash
helm upgrade --install spire spiffe/spire \
  --namespace spire --create-namespace \
  -f platform/spire/values.yaml

kubectl apply -f platform/spire/registration-entries.yaml
```

## Workload identity model

Each SPIFFE ID follows the pattern:

```
spiffe://eel.internal/k8s/<cluster>/<namespace>/<service-account>
```

Entry files in `entries/` bind a service account to a SPIFFE ID with attestor selectors:
- Kubernetes service account (all environments)
- TPM 2.0 attestation (production kit nodes)
- HSM hardware binding (HSM signer only)

DR-related entries (`federation-dr-primary.yaml`, `federation-dr-mirror.yaml`) establish trust federation between the primary and mirror clusters.

## Entry format

```yaml
spiffeId: spiffe://eel.internal/k8s/primary/eel-rts/rts-svc
parentId: spiffe://eel.internal/k8s/primary/spire/agent
selectors:
  - type: k8s
    value: "ns:eel-rts"
  - type: k8s
    value: "sa:rts-svc"
ttl: 3600
```

## Verify SVID issuance

```bash
# Check agent is running and has SVIDs:
kubectl exec -n spire spire-agent-<pod> -- /opt/spire/bin/spire-agent api fetch x509

# List all registered entries:
kubectl exec -n spire spire-server-0 -- /opt/spire/bin/spire-server entry show
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind dev: disk attestor, reduced TTLs |

## Operator responsibilities

- Service accounts must match `entries/<svc>.yaml` exactly. A renamed SA loses its SVID.
- SVID TTL is 1 hour; rotation is automatic. No manual certificate renewal.
- Trust bundles for the DR mirror are in `federation-dr-mirror.yaml`; update when the mirror cert rotates.

## Related docs

- `platform/cilium/README.md`: Cilium uses SVIDs for identity-based network policy
- `tools/iam-seed/README.md`: `iam-seed spire` registers entries programmatically
- `docs/runbooks/svid-debug.md`: debugging SVID issuance failures
