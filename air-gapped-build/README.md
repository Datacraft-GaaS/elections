# air-gapped-build

Offline build environment for eel. All dependencies are mirrored through a local Sonatype
Nexus OSS proxy. Once the Nexus cache is populated, builds run with zero outbound internet
access. Per ADR-0006.

## Why air-gapped

The Commission's secure build environment has no outbound internet. Supply-chain integrity
requires that every package pulled during a build is the same one that was reviewed and
pinned. Nexus acts as a controlled mirror: packages enter only through an explicit
population step, checksums are enforced strictly, and the mirror is snapshotted before
a release build.

## Full offline build recipe

```
1. populate Nexus mirror (one-time, on internet-connected machine)
   -> nexus-config/nexus.groovy + pinned-repos.yaml
2. snapshot Nexus mirror (airgap the machine)
3. build: air-gapped-build/build-image/build.sh <nix-attr>
4. collect SBOM and timestamps
5. sign and publish to sbom-cache/
```

## Contents

| Directory | Description |
|-----------|-------------|
| `nexus-config/` | Nexus bootstrap script and pinned repository definitions |
| `build-image/` | Offline `nix build` wrapper script and container image definition |
| `sbom-cache/` | SBOM artifacts produced by builds; input to hash manifests |
| `timestamps/` | RFC 3161 trusted timestamps for build artifacts |

## Usage

```bash
# 1. Bootstrap Nexus repositories (run once on internet-connected machine)
cd air-gapped-build/nexus-config
curl -u admin:$NEXUS_PASS \
  -H "Content-Type: application/json" \
  http://nexus.internal:8081/service/rest/v1/script \
  -d @nexus-bootstrap.json
curl -u admin:$NEXUS_PASS -X POST \
  http://nexus.internal:8081/service/rest/v1/script/eel-nexus-bootstrap/run

# 2. Run an offline build
export NEXUS_URL=http://nexus.internal:8081
bash air-gapped-build/build-image/build.sh packages.x86_64-linux.eel-rts-svc

# 3. Integrate with make reproduce
NIX_OFFLINE=1 make reproduce BUNDLE=/bundles/eel-release.tar.gz
```

## Config and environment

| Variable | Default | Description |
|----------|---------|-------------|
| `NEXUS_URL` | `http://nexus.internal:8081` | Nexus OSS base URL |
| `NEXUS_PASS` | required | Nexus admin password (from vault) |
| `NEXUS_CA` | optional | CA bundle for Nexus TLS if using HTTPS |
| `NIX_OFFLINE` | `0` | Pass `1` to force `--offline` Nix builds |

## Dependencies

- Sonatype Nexus OSS >= 3.60 running at `$NEXUS_URL`
- Nix >= 2.22 with flakes enabled
- Docker or Podman for the build-image container (optional; can run bare-metal)

## Test and verify

```bash
# Confirm Nexus repos are healthy
curl -s http://nexus.internal:8081/service/rest/v1/repositories \
  | jq '.[].name' | grep eel-

# Test an offline build (small package)
bash air-gapped-build/build-image/build.sh packages.x86_64-linux.eel-crypto-rs
```

## Related docs

- `air-gapped-build/nexus-config/README.md`: Nexus setup detail
- `air-gapped-build/build-image/README.md`: build script and container
- `air-gapped-build/sbom-cache/README.md`: SBOM storage
- `air-gapped-build/timestamps/README.md`: RFC 3161 timestamping
- `reproducibility/README.md`: how air-gapped builds feed into reproduction
- `docs/adr/0006-reproducible-builds.md`: design ADR

## Limitations

- Nexus must be pre-populated before airgapping; missing packages will cause build failure
  with no fallback.
- Package pinning is per the `pinned-repos.yaml` versions; security patches require an
  explicit pin update, Nexus re-population, and re-attestation.
- The build container does not include a Rust cross-compiler; ARM builds require a
  separate cross-compilation pass documented in `build-image/cross-compile.md`.
