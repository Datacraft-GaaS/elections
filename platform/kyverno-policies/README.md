# platform/kyverno-policies/

Kyverno admission policies for the eel IEMS cluster. All policies are `ClusterPolicy` resources in enforce mode.

## Policies

| File | Rule | Effect |
|---|---|---|
| `require-signed-images.yaml` | All images must carry a cosign signature from the eel Harbor registry | Deny unsigned images |
| `forbid-privileged.yaml` | No container may run with `privileged: true` or `allowPrivilegeEscalation: true` | Deny |
| `forbid-hostpath.yaml` | No `hostPath` volumes permitted (except SPIRE agent socket, explicitly allowed) | Deny |
| `forbid-nodeport.yaml` | No `NodePort` Services; all external traffic via Envoy Gateway | Deny |
| `forbid-latest-tag.yaml` | Image tags must be a digest or semver; `:latest` is blocked | Deny |
| `require-probes-and-limits.yaml` | All pods must declare `readinessProbe`, `livenessProbe`, CPU and memory `limits` | Deny |
| `require-otel-annotations.yaml` | All pods must carry `prometheus.io/scrape`, `prometheus.io/port` annotations | Warn (enforced at G2) |
| `require-svid-label.yaml` | All pods in `eel-*` namespaces must carry `spiffe.io/spiffeid: "true"` | Deny |

## Install

```bash
# Install Kyverno operator first:
helm upgrade --install kyverno kyverno/kyverno \
  --namespace kyverno --create-namespace

# Apply all policies:
kubectl apply -f platform/kyverno-policies/
```

## Validate without applying

```bash
kyverno validate platform/kyverno-policies/require-signed-images.yaml
kyverno validate platform/kyverno-policies/
```

## Testing policies

```bash
# Kyverno CLI test (requires test cases in each policy YAML):
kyverno test platform/kyverno-policies/

# Apply a violating manifest to verify rejection:
kubectl apply -f docs/test-fixtures/unsigned-image-pod.yaml
# Expected: admission webhook denied
```

## Election-day mode

During the election-day freeze window, `platform/election-day-mode/freeze-window-policy.yaml` adds a Kyverno `ClusterPolicy` that blocks ArgoCD sync for labelled election-critical Applications. This policy is applied separately and outside this directory.

## Operator responsibilities

- Policies are in enforce mode from Wave 1 onwards. Test new images in a kind cluster before pushing to production.
- The cosign verification key is stored in `eel-vault` via OpenBao and mounted as a Kyverno secret.
- Do not add `audit` mode policies without a tracking issue and expiry date.

## Related docs

- `platform/harbor/README.md`: image registry and cosign signing
- `platform/election-day-mode/README.md`: freeze-window policy
- `docs/runbooks/policy-exception.md`: process for a time-limited policy exception
