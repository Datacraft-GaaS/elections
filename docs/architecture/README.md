# eel System Architecture

This directory provides per-service architecture pages, shared-package overviews, and cross-cutting system diagrams for the eel (Integrated Election Management System).

## Services (18)

| Service | Architecture Page | Primary Modules |
|---|---|---|
| transparency | [services/transparency.md](services/transparency.md) | M14, M22 |
| boundary | [services/boundary.md](services/boundary.md) | M18 |
| rts | [services/rts.md](services/rts.md) | M08, M09, M10, M51, M63 |
| bvr | [services/bvr.md](services/bvr.md) | M02, M35, M36, M53 |
| abis | [services/abis.md](services/abis.md) | M03 |
| bvi | [services/bvi.md](services/bvi.md) | M06, M07 |
| biometric-edge | [services/biometric-edge.md](services/biometric-edge.md) | M01, M05, M24 |
| portal | [services/portal.md](services/portal.md) | M11, M52, M64, M68 |
| ecc | [services/ecc.md](services/ecc.md) | M15, M16, M23 |
| queue | [services/queue.md](services/queue.md) | M19 |
| inventory | [services/inventory.md](services/inventory.md) | M20, M39, M65 |
| ai-platform | [services/ai-platform.md](services/ai-platform.md) | M17, M26, M28, M44, M61 |
| field-mobile | [services/field-mobile.md](services/field-mobile.md) | M13, M29-M34 |
| vprs | [services/vprs.md](services/vprs.md) | M04 |
| crms | [services/crms.md](services/crms.md) | M12, M45, M46, M47, M67 |
| governance | [services/governance.md](services/governance.md) | M21, M40, M41, M48-M50, M56-M59, M62, M66 |
| training | [services/training.md](services/training.md) | M37, M38, M42, M43, M60 |
| support-admin | [services/support-admin.md](services/support-admin.md) | M25, M27, M54, M55, M69 |

## Shared Packages

| Package | Purpose |
|---|---|
| eel-auth-{rs,ex,py} | SPIFFE SVID validation, RBAC enforcement |
| eel-crypto-rs | HSM abstraction, signing, key-derivation |
| eel-events-{rs,ex,py} | CloudEvents v1 schema, outbox helpers |
| eel-eventstore-{rs,ex,py} | Append-only event log over PostgreSQL |
| eel-hlc-{rs,ex,py} | Hybrid logical clock for cross-service ordering |
| eel-loadshedding-{rs,ex,py} | Load-shedding circuit breaker |
| eel-observability-{rs,ex,py} | Structured logging, metrics, trace propagation |
| eel-outbox-{rs,ex,py} | Transactional outbox pattern |
| eel-pii-{rs,ex,py} | PII classification, redaction, tier enforcement |
| eel-resilience-{rs,ex,py} | Retry, bulkhead, timeout decorators |
| eel-secrets-{rs,ex,py} | OpenBao secret lease management |
| eel-api-server-rs | Axum HTTP server baseline with middleware stack |
| openapi-contracts | Per-service OpenAPI 3.1 specifications |
| event-schemas | CloudEvents JSON Schema registry |
| kit-sdk-flutter | Flutter SDK for field kit devices |
| design-system | Shared UI tokens and components |
| data-dictionary | Canonical field names and PII tiers |
| authz-schema | SpiceDB schema for all services |
| api-clients | Generated clients for Rust, Python, Elixir, Dart |

## Platform

Kubernetes (k3s at edge, full at primary/mirror) managed by ArgoCD. See `platform/` for Helm values and ArgoCD ApplicationSets. Deployment topology: [cross-cutting/deployment-topology.md](cross-cutting/deployment-topology.md).

## Edge

Five regional edge clusters. See `edge/` and [cross-cutting/deployment-topology.md](cross-cutting/deployment-topology.md).

## Integrations

External system adapters under `integrations/`. Key integrations: IPRS, NIDA, KRA, EACC, PPDT, NLC, KNBS, Daraja, M-Pesa B2C, IFMIS, eCourt, GovPrinter, NCIC, SAML identity providers.

## Cross-cutting Diagrams

- [Event flow](cross-cutting/event-flow.md)
- [Trust model](cross-cutting/trust-model.md)
- [Data tiers](cross-cutting/data-tiers.md)
- [Deployment topology](cross-cutting/deployment-topology.md)
