# Production Environment

Production runs all 7 sites. The `eel-election-day-mode` ConfigMap is deployed here and toggled
by the Chair HSM-signed toggle procedure described in
`platform/election-day-mode/toggle-runbook.md`.

## Overlay apply order

1. `kustomization.yaml` (root) — imports platform and service manifests with full Enforce policy.
2. `overlays/primary/` — primary cluster, 3 replicas, election-day-mode flag patch.
3. `overlays/mirror/` — mirror cluster, 3 replicas, election-day-mode flag patch.
4. `overlays/edge-{nairobi,mombasa,kisumu,eldoret,garissa}/` — edge clusters, 2 replicas each.

## election-day-mode flag

The `eel-election-day-mode` ConfigMap `data.election-day-mode-enabled` defaults to `"false"`.
During T-1 through T+1, the toggle runbook sets it to `"true"` with dual SVID sign-off. The
Kyverno freeze policy blocks sync-wave-1 Application syncs while the flag is active.

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
