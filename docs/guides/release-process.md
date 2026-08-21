# Release Process

## Overview (ADR-0022)

Releases are built deterministically, signed, attested, and deployed via ArgoCD blue-green promotion. No release may reach production without a reproducible build verification and SLO gate pass.

## Steps

### 1. Build

```sh
just build-all
```

Builds all service images via Nix (`nix build .#<service>-image`). Images are content-addressed; the same source always produces the same OCI layer hashes on the same architecture.

### 2. Sign

Images are signed with Cosign using the release key stored in the HSM (`eel-release-signing` partition):

```sh
just sign-images
```

The signing step produces a Sigstore signature bundle stored in the transparency log (`tlog`). The log entry URL is recorded in `reproducibility/hash-manifests/`.

### 3. Publish

Signed images are pushed to the internal OCI registry. The registry enforces signature verification on pull:

```sh
just publish-images <registry-url>
```

### 4. Deploy

ArgoCD watches the `environments/staging/` and `environments/pre-prod/` directories. To promote from staging to pre-prod:

```sh
just promote staging pre-prod
```

This updates the image tags in `environments/pre-prod/values.yaml` and commits the change. ArgoCD detects the commit and triggers the Argo Rollout.

### 5. Verify reproducibility

After pre-prod deploy, run the reproducibility check:

```sh
just make-reproduce <bundle-path>
```

This rebuilds images from the pinned flake, re-derives the tally and Merkle root from the bundle, and runs `diffoscope` to confirm bit-identical output on both x86_64 and aarch64. Gate G5 requires a clean `make reproduce` before election day.

### 6. Promote to production

Production promotion requires two approvals (tech lead + IEBC OIT representative). The Argo Rollout uses blue-green for stateful services and canary for portal-svc/vprs-svc. Rollback takes less than 15 min (ADR-0022 RTO).

## Election-day freeze

Before election-day mode is activated, schema migrations and non-critical background jobs are frozen. No release may be deployed during election-day mode without an emergency change process.

## Merkle Root

The Merkle root for each release bundle is recorded in `reproducibility/hash-manifests/`. The root links the git commit hash, all image digests, and the tally derivation to a single verifiable anchor. Observers can independently verify the root using `just verify-bundle <root>`.

## Related

- `reproducibility/make-reproduce/README.md` - reproducibility script documentation
- `air-gapped-build/README.md` - air-gapped build procedure for sensitive environments
- `compliance/ipr-handover/build-instructions.md` - GCC 15.4 build instructions
