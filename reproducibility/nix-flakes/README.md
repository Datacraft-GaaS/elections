# reproducibility/nix-flakes

Per-language Nix flake fragments for hermetic, reproducible eel builds. Each subdirectory
provides the Nix derivation logic for one language tier. The root `flake.nix` composes
these fragments into a single flake that produces all eel service images.

## Purpose

Nix is the build system that makes byte-for-byte reproducibility practical. Every input
(compiler version, library version, system dependency) is pinned via the flake lockfile.
Given the same `flake.lock`, any machine (with or without internet access, via the Nexus
proxy) produces the same output hash.

## Contents

| Directory | Language tier | Key toolchain |
|-----------|---------------|---------------|
| `rust/` | Rust services (`*-svc` crates) | `crane` + `rustToolchain` pinned via `rust-overlay` |
| `python/` | Python workers and FastAPI services | `uv2nix` or `poetry2nix` |
| `elixir/` | Elixir/Phoenix services | `mix2nix` |
| `dart/` | Flutter kit apps | `flutter.packages` from nixpkgs |
| `node/` | Astro sites and TS tooling | `node2nix` or `npmlock2nix` |

## Usage

```bash
# Build all eel services (long; use for full verification)
nix build .#eel-all

# Build a specific service image
nix build .#eel-rts-svc
nix build .#eel-portal-svc

# Build offline (requires Nexus proxy pre-populated)
export NEXUS_URL=http://nexus.internal:8081
nix build .#eel-rts-svc \
  --offline \
  --substituters "$NEXUS_URL/repository/eel-nix-cache-proxy/"

# Show all buildable outputs
nix flake show
```

## Config and environment

| File | Description |
|------|-------------|
| `flake.lock` | Pinned inputs for all flake fragments; commit every change |
| `rust/toolchain.toml` | Rust toolchain version pinned by `rustup` compatible spec |
| `python/requirements-lock.txt` | Locked PyPI deps for `uv2nix` derivations |
| `elixir/mix.lock` | Mix lockfile consumed by `mix2nix` |

To update a pinned input (do not do this casually; a change breaks reproducibility
attestation for all prior builds):

```bash
# Update a specific flake input
nix flake update nixpkgs --commit-lock-file

# Verify no hash changes in dependent derivations
nix build .#eel-rts-svc 2>&1 | grep "built\|copied"
```

## Dependencies

- Nix >= 2.22 with flakes enabled (`experimental-features = nix-command flakes`)
- `nixpkgs` pinned in `flake.lock`
- `crane` (Rust), `uv2nix` (Python), `mix2nix` (Elixir), pinned in `flake.lock`
- For offline builds: Nexus proxy at `$NEXUS_URL` with `eel-nix-cache-proxy` configured

## Test and verify

```bash
# Check flake integrity
nix flake check

# Verify Rust tier reproducibility (two builds, compare hashes)
nix build .#eel-rts-svc --out-link /tmp/build1
nix build .#eel-rts-svc --out-link /tmp/build2 --check
# If hashes differ, nix prints the diff and exits non-zero.

# Run derivation-level tests
nix build .#checks.x86_64-linux.all
```

## Related docs

- `reproducibility/README.md`: full reproducibility workflow
- `air-gapped-build/README.md`: how flake builds run against the Nexus proxy
- `docs/adr/0006-reproducible-builds.md`: ADR for Nix-based reproducibility

## Limitations

- Elixir/Erlang builds are near-reproducible but not byte-for-byte on all platforms due
  to timestamp embedding in BEAM files. Known non-determinism is documented in
  `elixir/KNOWN-NONDETERMINISM.md`.
- Dart/Flutter builds are reproducible only on the same Dart SDK version; SDK upgrades
  must be coordinated and re-attested.
- First build on a fresh machine without Nexus cache takes 45-90 minutes; cached builds
  take under 5 minutes per service.
