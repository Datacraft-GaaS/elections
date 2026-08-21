# Staging Environment

Staging runs primary and mirror sites only (no edge). Kyverno policies are in Audit mode.
ArgoCD control-plane replicas are reduced for capacity.

## Overlay apply order

1. Base kustomization (`kustomization.yaml`) — imports platform and service manifests.
2. `overlays/primary/` — replica count for primary cluster.
3. `overlays/mirror/` — replica count for mirror cluster.

## Sites in staging

| Site | Cluster | Purpose |
|---|---|---|
| primary | https://kubernetes.default.svc | Main staging workloads |
| mirror | https://mirror.eel.internal:6443 | Failover testing |

## Notes

- Kyverno validates but does not enforce (Audit mode) in staging.
- SPIRE trust domain: `eel.staging`.
- No election-day-mode flag in staging (flag is prod-only).
