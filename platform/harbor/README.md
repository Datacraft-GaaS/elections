# platform/harbor/

Harbor container registry for the eel IEMS. All service images are built in CI and pushed here. Kyverno enforces that only images from this registry (with a valid cosign signature) can run in the cluster.

## Contents

| File | Description |
|---|---|
| `harbor-values.yaml` | Helm values (S3-backed storage via MinIO, OIDC auth via Keycloak, cosign notary enabled) |
| `local/` | Kustomize overlay for kind (filesystem storage, no MinIO dependency) |

## Install

```bash
helm upgrade --install harbor harbor/harbor \
  --namespace harbor --create-namespace \
  -f platform/harbor/harbor-values.yaml
```

## Projects

| Project | Description | Public |
|---|---|---|
| `eel` | All production service images | No |
| `eel-platform` | Platform component images (overrides for airgap) | No |
| `eel-dev` | CI and dev images (not deployed to prod) | No |

## Image signing (cosign)

CI signs every pushed image with cosign using the `eel-code-signing` key from the HSM signer:

```bash
# Sign after push (done by CI pipeline):
cosign sign \
  --key env://COSIGN_PRIVATE_KEY \
  harbor.eel.internal/eel/rts-svc:v1.2.3@sha256:<digest>
```

Kyverno `require-signed-images.yaml` verifies the signature on admission using the public key stored as a Kyverno secret.

## Garbage collection

GC runs nightly at 02:00 UTC. Images not referenced by any running pod or the last 10 tags are pruned. Do not push to Harbor without tagging; untagged blobs are collected within 24 hours.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: filesystem storage, self-signed TLS, no OIDC |

## Operator responsibilities

- OIDC integration uses the `eel-realm` in Keycloak. Rotate the client secret in OpenBao when it expires (90-day TTL).
- Registry mirrors for air-gap environments are declared in `harbor-values.yaml` under `proxy.remoteProxies`.
- Replication rules for the DR mirror registry are managed by Harbor itself; check `harbor.mirror.eel.internal` replication jobs weekly.

## Related docs

- `platform/kyverno-policies/README.md`: `require-signed-images` policy
- `platform/keycloak/README.md`: OIDC client for Harbor auth
- `platform/minio/README.md`: S3 backend for Harbor blobs
