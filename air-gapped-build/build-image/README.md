# air-gapped-build/build-image

Offline `nix build` workflow for eel, integrated with `make reproduce`. Runs `nix build`
in `--offline` mode with all build tool network fetches redirected to the local Nexus proxy.

## Overview

The build-image environment runs `nix build` with all external network access blocked.
All dependencies must be present in the Nexus proxy cache or the local Nix store before
the build starts. The `build.sh` script sets the relevant proxy environment variables for
Cargo, pip, npm, Hex, and Nix substituters so that any build hook that attempts a network
fetch uses the local mirror.

## Prerequisites

1. Nexus OSS running at `$NEXUS_URL` (default: `http://nexus.internal:8081`).
2. Nexus repositories provisioned via `nexus-config/nexus.groovy` and `pinned-repos.yaml`.
3. Nix >= 2.22 with flakes enabled.
4. The eel flake pinned inputs pre-fetched into the Nix store:

```bash
nix flake update --commit-lock-file
nix flake archive --to "$NEXUS_URL/repository/eel-nix-cache-proxy"
```

## Contents

| File | Description |
|------|-------------|
| `build.sh` | Offline build wrapper: sets proxy env vars, calls `nix build --offline` |
| `Containerfile` | OCI container image definition for the build environment |
| `Dockerfile` | Docker-compatible alias of `Containerfile` |

## Running an offline build

```bash
cd air-gapped-build/build-image
export NEXUS_URL=http://nexus.internal:8081
bash build.sh packages.x86_64-linux.eel-rts-svc

# Build all services (slow; use for release verification)
bash build.sh packages.x86_64-linux.eel-all
```

The script sets `--offline` so no external network access occurs. The build output path
is printed on success.

## Container-based build

```bash
# Build the builder image
podman build -t eel-builder:latest -f Containerfile .

# Run an offline build inside the container
podman run --rm \
  -v "$(pwd)/../..:/workspace:ro" \
  -e NEXUS_URL=http://nexus.internal:8081 \
  eel-builder:latest \
  bash build.sh packages.x86_64-linux.eel-portal-svc
```

## Integration with make reproduce

`make reproduce` calls `scripts/reproduce.sh`, which invokes this build for each image
in the hash manifest. To run that step offline:

1. Pre-populate the Nix store via this build-image workflow.
2. Set `NIX_OFFLINE=1` before calling `make reproduce`; `reproduce.sh` passes `--offline`
   to nix.

```bash
NIX_OFFLINE=1 make -C reproducibility/make-reproduce \
  reproduce BUNDLE=/bundles/eel-release.tar.gz
```

## Config and environment

| Variable | Default | Description |
|----------|---------|-------------|
| `NEXUS_URL` | `http://nexus.internal:8081` | Nexus base URL for all proxy env vars |
| `NEXUS_CA` | unset | CA bundle path if Nexus uses TLS |
| `CARGO_REGISTRIES_CRATES_IO_PROTOCOL` | `sparse` | Set by `build.sh` |
| `PIP_INDEX_URL` | derived from `$NEXUS_URL` | Set by `build.sh` |
| `NPM_CONFIG_REGISTRY` | derived from `$NEXUS_URL` | Set by `build.sh` |
| `NIX_SUBSTITUTERS` | derived from `$NEXUS_URL` | Set by `build.sh` |

## Dependencies

- Nix >= 2.22 with flakes enabled
- Nexus OSS with `eel-nix-cache-proxy`, `eel-crates-io-proxy`, `eel-pypi-proxy`,
  `eel-npm-proxy` provisioned (see `nexus-config/`)

## Test and verify

```bash
# Minimal smoke test: build a small Rust crate
bash build.sh packages.x86_64-linux.eel-crypto-rs
echo "Exit code: $?"

# Confirm no outbound requests were made (run with strace in the container)
strace -f -e trace=network \
  bash build.sh packages.x86_64-linux.eel-crypto-rs 2>&1 \
  | grep -v "127.0.0.1\|nexus.internal" \
  | grep connect
# Expected: no lines (no outbound connections outside localhost/Nexus)
```

## Related docs

- `air-gapped-build/README.md`: overall offline build overview
- `air-gapped-build/nexus-config/README.md`: Nexus setup
- `reproducibility/make-reproduce/README.md`: pipeline that calls this build

## Limitations

- The script does not verify that Nexus has all required packages before starting. A
  missing package causes a build failure mid-way; pre-warm the cache completely.
- ARM (aarch64) cross-compilation is not handled by `build.sh`; see
  `cross-compile.md` for that workflow.
- Container builds using `Containerfile` require Podman 4+ or Docker 24+.
