# reproducibility/make-reproduce

`make reproduce` rebuilds eel OCI images from a pinned Nix flake and re-derives the tally Merkle root from bundle events, producing a diffoscope diff on any mismatch.

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Nix | 2.22+ | Hermetic image builds |
| cosign | 2.2+ | SBOM signature verification |
| diffoscope | 264+ | Layer-level diffs |
| jq | 1.6+ | JSON processing |
| cargo | 1.78+ | Build `replay-tally-rs` |

## Quick start

```sh
# Build the tally replayer once
cd bin/replay-tally-rs && cargo build --release && cd -

# Run the full pipeline against the synthetic fixture
make reproduce BUNDLE=reproducibility/make-reproduce/fixtures/mini-bundle

# Or against a real bundle path/URL
make reproduce BUNDLE=/path/to/bundle.tar.gz
make reproduce BUNDLE=https://bundles.example/eel-wave4-bundle.tar.gz
```

## Targets

| Target | Description |
|--------|-------------|
| `make reproduce BUNDLE=<path\|url>` | Full pipeline: fetch, SBOM verify, Nix rebuild, sha256 compare, diffoscope on mismatch |
| `make verify-images IMAGES="img-a img-b"` | Rebuild specific images via Nix, emit hash report |
| `make replay-tally BUNDLE=<path>` | Replay bundle events, derive Merkle root, compare to declared root |
| `make diff-artefact A=<path> B=<path>` | Run diffoscope with JSON+HTML output |
| `make clean-repro` | Remove `.repro-reports/` |

## Pipeline stages (reproduce.sh)

1. Fetch bundle (URL or local path), extract archive.
2. Verify SBOM `.sig` files with cosign (warns if cosign key not configured).
3. For each `kind=image` artefact in `manifest.json`: run `nix build .#<build_recipe>`.
4. Compute sha256 of Nix output, compare to manifest.
5. On mismatch: unpack OCI layers and invoke diffoscope, write JSON report.
6. Write overall JSON report to `.repro-reports/`.

## Tally replay (replay-tally.sh / replay-tally-rs)

Reads `events.jsonl` from the bundle, hashes each event using BLAKE3 over RFC 8785 canonical JSON, chains hashes into a Merkle root, and compares to `manifest.json:.tally_root`.

The Rust binary is a standalone crate (empty `[workspace]` stanza) that does not join the root Cargo workspace.

```sh
cd bin/replay-tally-rs
cargo build --release
cargo test
```

## Multi-arch verification

```sh
bash scripts/multi-arch-verify.sh packages.x86_64-linux.eel-vprs
```

Builds the given Nix attr on x86_64-linux, aarch64-linux, and aarch64-darwin. Skips unavailable platforms with a warning. Asserts all available hashes are identical.

## Docker hermetic environment

```sh
export BUNDLE_PATH=/path/to/bundle
export REPORT_DIR=/tmp/reports
docker compose up --build
```

The container has pinned Nix, cosign, diffoscope, b3sum, and jq. `network_mode: none` ensures no external fetches at reproduce time.

## Expected outputs

A successful `make reproduce` writes `.repro-reports/reproduce-<timestamp>.json`:

```json
{
  "schema_version": "1",
  "bundle": "mini-bundle",
  "timestamp": "2026-08-01T00:00:00Z",
  "overall": "ok",
  "artefacts": [
    {"name": "eel-fixture-image", "kind": "image", "status": "skipped"}
  ]
}
```

Status values: `ok` (hash match), `mismatch` (hash differs, see diff report), `no-build` (Nix unavailable), `skipped` (non-image artefact).

## Failure semantics

- Any `mismatch` causes `reproduce.sh` to exit non-zero after emitting diffoscope reports for all mismatched artefacts.
- A root mismatch in `replay-tally.sh` exits non-zero with the computed vs declared roots in the report.
- Missing tools (nix, diffoscope, cosign) produce warnings, not hard failures, except for the replay binary which is required.

## Running tests

```sh
bash tests/test_reproduce.sh
```
