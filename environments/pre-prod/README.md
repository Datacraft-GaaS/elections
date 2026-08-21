# Pre-Production Environment

Pre-prod is the last gate before production. It runs all 7 sites (primary, mirror, 5 edge
clusters) and is reserved for 5x load testing.

## Overlay apply order

1. `kustomization.yaml` (root) — imports platform and service manifests with Enforce policy.
2. `overlays/primary/` — primary cluster, 3 replicas, `load-profile: 5x`.
3. `overlays/mirror/` — mirror cluster, 3 replicas, `load-profile: 5x`.
4. `overlays/edge-{nairobi,mombasa,kisumu,eldoret,garissa}/` — edge clusters, 2 replicas each, `load-profile: 5x`.

## 5x load-test profile

Every Deployment in pre-prod carries `load-profile: 5x`. The load-test harness reads this label
to scale its injector threads accordingly.

## Sites

| Site | Cluster | Replicas |
|---|---|---|
| primary | https://kubernetes.default.svc | 3 |
| mirror | https://mirror.eel.internal:6443 | 3 |
| edge-nairobi | https://edge-nairobi.eel.internal:6443 | 2 |
| edge-mombasa | https://edge-mombasa.eel.internal:6443 | 2 |
| edge-kisumu | https://edge-kisumu.eel.internal:6443 | 2 |
| edge-eldoret | https://edge-eldoret.eel.internal:6443 | 2 |
| edge-garissa | https://edge-garissa.eel.internal:6443 | 2 |

## Notes

- Kyverno: signed-image check is Enforce; other policies are Audit.
- No election-day-mode ConfigMap deployed in pre-prod (prod-only).
