# platform/envoy-gateway/

Envoy Gateway installation and route configuration for the eel IEMS. Acts as the north-south ingress/egress gateway for all external-facing services.

## Contents

| File | Description |
|---|---|
| `install.yaml` | Envoy Gateway install manifest (Gateway API CRDs + controller) |
| `local/` | Kustomize overlay for kind (NodePort listener) |

Route and `HTTPRoute` resources live in each service's `deploy/` overlay, not here.

## Install

```bash
kubectl apply -f platform/envoy-gateway/install.yaml
# Wait for controller:
kubectl wait --namespace envoy-gateway-system \
  deployment/envoy-gateway --for=condition=Available --timeout=90s
```

## Gateway classes

| Class | Use |
|---|---|
| `eel-external` | Internet-facing routes (portal, media, public) |
| `eel-internal` | Cluster-internal API routes |
| `eel-admin` | Admin surfaces (restricted source CIDRs) |

## Traffic routing during DR failover

During `eel-dr-failover execute`, the failover tool sets Envoy Gateway route weights:
- Phase 1: primary 100%, mirror 0% (normal)
- Phase 2: primary 0%, mirror 0% (quiesce)
- Phase 3: primary 0%, mirror 100% (failover complete)

This is driven by patching `HTTPRoute` weight fields via kubectl, not by editing this directory.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single `NodePort` listener, no LoadBalancer dependency |

## Operator responsibilities

- TLS termination uses certificates from cert-manager `ClusterIssuer: eel-internal-ca`.
- Rate limiting is configured per route in the service `HTTPRoute`; do not add global rate limits here without an ADR.
- Access logs flow to the OTel Collector via the Envoy access log filter.

## Related docs

- `platform/kong/README.md`: Kong API gateway (internal tier, separate from Envoy)
- `platform/cert-manager/README.md`: TLS certificate issuance
- `tools/dr-failover/README.md`: failover tool that patches route weights
