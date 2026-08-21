# ApplicationSets: generated

The 19 ApplicationSet manifests in the parent directory (`platform/argocd/applicationsets/`) were
produced by `tools/appset-gen` (Rust, WP-1.9) from each service's
`services/<svc>/deploy/service.yaml`.

## Regenerate

```bash
cd tools/appset-gen
cargo run -- \
  --services-root ../../services \
  --out ../../platform/argocd/applicationsets
```

The tool is idempotent: re-running it produces byte-identical output for unchanged service
descriptors.

## Rollout strategy annotation

| Annotation | Value | Services |
|---|---|---|
| `argocd.argoproj.io/sync-wave` | `"1"` | rts-svc, bvr-svc, abis-svc, bvi-svc, biometric-edge-svc, boundary-svc, transparency-svc, portal-svc |
| `argocd.argoproj.io/sync-wave` | `"2"` | all others |
| `eel.internal/rollout-strategy` | `blue-green` | wave-1 services |
| `eel.internal/rollout-strategy` | `canary` | wave-2 services |

## Site generators

Each ApplicationSet expands to 7 Applications (primary, mirror, edge-nairobi, edge-mombasa,
edge-kisumu, edge-eldoret, edge-garissa) via a list generator. The source path for each
Application is `services/<svc>/deploy/overlays/<site>`.
