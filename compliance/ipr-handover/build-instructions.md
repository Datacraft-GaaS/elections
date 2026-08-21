# Build Instructions

End-to-end reproducible build procedure for the eel system. References `reproducibility/make-reproduce/README.md` for the verification step.

## Prerequisites

- Nix 2.18+ with flakes enabled
- 16 GB RAM minimum; 32 GB recommended for full parallel build
- Access to the internal container registry (for publish step)
- HSM access for signing (hardware token or PKCS#11 emulator for test builds)

## Step 1: Enter the Nix development shell

```sh
nix develop
```

This pins all build tools to the versions in `flake.lock`. Do not use system-installed tools for reproducible builds.

## Step 2: Build all service images

```sh
just build-all
```

Internally this runs `nix build .#<service>-image` for all 18 services. Images are content-addressed; the same source commit always produces the same OCI layer digests on the same CPU architecture.

For a single service:

```sh
nix build .#rts-svc-image
```

## Step 3: Run the test suite

```sh
just test
```

All tests in `tests/ci/` must pass. The build is not considered clean if any test fails.

## Step 4: Sign images

Images are signed using Cosign with the HSM-backed release key:

```sh
just sign-images
```

This requires the `eel-release-signing` HSM partition to be unlocked. For air-gapped builds, follow `air-gapped-build/README.md` instead.

Signing produces:
- A Cosign signature bundle for each image
- A Sigstore transparency log entry URL recorded in `reproducibility/hash-manifests/`

## Step 5: Publish images

```sh
just publish-images <registry-url>
```

Pushes all signed images and their SBOM attestations to the OCI registry. The registry enforces signature verification on pull; unsigned images are rejected.

## Step 6: Deploy

```sh
just promote staging pre-prod
```

Updates image tags in `environments/pre-prod/` and triggers ArgoCD reconciliation. See `docs/guides/release-process.md` for the full deployment procedure.

## Step 7: Verify reproducibility

```sh
just make-reproduce <bundle-path>
```

Rebuilds images from the pinned flake, re-derives the tally and Merkle root from the election bundle, and runs `diffoscope` to confirm bit-identical output. This step is required by Gate G5 before election day.

For the full procedure see `reproducibility/make-reproduce/README.md`.

## Air-gapped build

For secure environments without internet access, see `air-gapped-build/README.md`. The air-gapped procedure uses a pre-fetched Nix store tarball and an offline Nexus registry mirror.

## Build artefacts

| Artefact | Location | Description |
|---|---|---|
| OCI images | Internal OCI registry | Content-addressed service images |
| SBOM attestations | OCI registry (as attestations) | SPDX 2.3 SBOMs per image |
| Hash manifests | `reproducibility/hash-manifests/` | Per-release image digests and Merkle root |
| Tlog entries | Sigstore public instance | Signature transparency log URLs |

## Troubleshooting

- `nix build` fails with IFD error: run `nix flake update` and retry once; do not update `flake.lock` on main without a review.
- Image digest mismatch: ensure you are building from a clean `git status` with no local modifications.
- HSM unavailable for signing: use `just sign-images --dry-run` to validate without signing; alert the release manager.
