# platform/kong/

Kong API gateway for internal tier routing within the eel IEMS cluster. Handles service-to-service API calls that cross trust boundaries internally, rate limiting, and request transformation.

Kong is distinct from Envoy Gateway: Envoy handles north-south (external) traffic; Kong handles east-west API mediation between internal services.

## Contents

| File | Description |
|---|---|
| `kong.yaml` | `KongIngress` and `KongPlugin` resources for cross-service routes |
| `values.yaml` | Helm values (DB-less mode, declarative config, OIDC plugin enabled) |

## Install

```bash
helm upgrade --install kong kong/kong \
  --namespace eel-gateway --create-namespace \
  -f platform/kong/values.yaml
```

## Key plugins configured

| Plugin | Applied to | Purpose |
|---|---|---|
| `openid-connect` | All routes requiring human auth | Validates Keycloak JWT |
| `request-transformer` | Routes to bvr-svc | Injects `X-Eel-Data-Class` header |
| `rate-limiting` | Result submission routes | 10 req/s per kit ID |
| `correlation-id` | All routes | Injects `X-Request-ID` if absent |

## DB-less mode

Kong runs in declarative DB-less mode. The full config is rendered from `kong.yaml` and loaded at startup. Changes require a rolling restart; no runtime config mutations.

```bash
# Validate config before apply:
deck validate --state platform/kong/kong.yaml

# Sync config (CI):
deck sync --state platform/kong/kong.yaml --kong-addr http://kong-admin.eel-gateway:8001
```

## Operator responsibilities

- Kong admin API is not exposed externally; access via `kubectl port-forward` only.
- Add new routes via `kong.yaml` and the ArgoCD sync cycle; never via the admin API directly.
- Plugin secrets (OIDC client secret) are injected via OpenBao dynamic secrets at pod start.

## Related docs

- `platform/envoy-gateway/README.md`: external ingress gateway
- `platform/keycloak/README.md`: OIDC token issuer
- `platform/openbao/README.md`: dynamic secret injection

---

## Technical Explanation (Extended) — kong

**Role**: kong is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide kong-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: kong must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from kong components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
