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

---

## Technical Explanation (Extended)

**Role**: Envoy Gateway provides HTTP/gRPC ingress for EEL — handles TLS termination, load balancing, routing, rate limiting, and audit logging for all external traffic.

**Purpose**: Secure ingress with audit chain (every request logged with audit reference), TLS termination (certificates from cert-manager), rate limiting (DDoS protection per ADR 0010), and M5 audit chain integration.

**No stubs**: `install.yaml` has real Envoy Gateway install manifest; `gateway.yaml` has real Gateway API resources; `values.yaml` has real Helm values.

**M5 Integration**: Every ingress request produces audit event (request ID, auditId via request header, proofHash of request payload, correlation_key for audit chain verification). Rate limits trigger Falco ALERT + audit_service event + Wazuh ALERT (rule id 100710 / eel-net-002). TLS handshake failures trigger Wazuh ALERT (rule id 100713 / eel-net-004).

**References**: ADR 0004 (Platform — ingress), ADR 0010 (Edge/DDoS — rate limit + TLS verification), ADR 0019 (Edge topology — edge site ingress routing), M5-SMS-USB-MANIFEST (M5 ingress audit chain), Elections Act §34 (secure ingress for form submission), NIST SP 800-53 SC-8 (Communications security — TLS termination).
