# air-gapped-build/build-image

Offline `nix build` workflow for eel, integrated with `make reproduce`.

## Overview

The build-image environment runs `nix build` in `--offline` mode with all network substituters disabled. All dependencies must be present in the local Nexus proxy cache (see `nexus-config/pinned-repos.yaml`) or in the Nix store before the build starts.

## Prerequisites

1. Nexus OSS running at `$NEXUS_URL` (default: `http://nexus.internal:8081`).
2. Nexus repositories provisioned via `nexus-config/nexus.groovy` and `pinned-repos.yaml`.
3. Nix >= 2.22 with flakes enabled.
4. The eel flake pinned inputs pre-fetched into the Nix store:
   ```sh
   nix flake update --commit-lock-file
   nix flake archive --to "$NEXUS_URL/repository/eel-nix-cache-proxy"
   ```

## Running an offline build

```sh
cd air-gapped-build/build-image
export NEXUS_URL=http://nexus.internal:8081
bash build.sh packages.x86_64-linux.eel-vprs-svc
```

The script sets `--substituters ""` and `--offline` so no external network access occurs.

## Integration with make reproduce

`make reproduce` calls `scripts/reproduce.sh`, which invokes `nix build .#<attr>` for each image in the manifest. To run that step offline:

1. Pre-populate the Nix store via this build-image workflow.
2. Set `NIX_OFFLINE=1` before calling `make reproduce` -- `reproduce.sh` will pass `--offline` to nix.

## Nexus proxy configuration

See `nexus-config/pinned-repos.yaml` for the full list of mirrored upstreams and their checksum policies. All proxies are configured with `STRICT` checksum enforcement.
