# reproducibility

M14 make-reproduce toolkit for the eel election platform. Every build artifact can be
independently reconstructed from source and compared byte-for-byte against the deployed
binary. This supports the tender requirement for verifiable, tamper-evident software.

## Why it matters

An election platform must be auditable. The Commission, accredited observers, and
independent technical experts must be able to confirm that what runs on election day
matches the reviewed source code. Nix hermetic builds plus transparency log attestations
make this provable rather than asserted.

## Full workflow

```
fetch bundle -> nix build -> diffoscope compare -> replay-tally -> tlog attestation
```

1. Fetch the release bundle (OCI images, binaries, SBOM, hash manifest, tlog entries).
2. Rebuild every artifact from source using the pinned Nix flake.
3. Compare rebuilt artifacts against the bundle using `diffoscope`; expect zero diffs.
4. Replay the `events.jsonl` tally log to re-derive the BLAKE3 Merkle root and compare
   against the `tally_root` field in the hash manifest.
5. Verify the hash manifest signature against the transparency log entry.

## Contents

| Directory | Description |
|-----------|-------------|
| `make-reproduce/` | Makefile-driven pipeline: reproduce, verify-images, replay-tally, diff-artefact |
| `hash-manifests/` | Schema, validation, and example manifests for build artifacts |
| `nix-flakes/` | Per-language Nix flake fragments (rust, python, elixir, dart, node) |

## Quick start

```bash
# Full reproduction pipeline against a release bundle
cd reproducibility/make-reproduce
make reproduce BUNDLE=/path/to/eel-release-2027-08-09.tar.gz

# Verify specific OCI images by Nix hash
make verify-images IMAGES="eel-rts-svc eel-bvi-svc eel-portal-svc"

# Replay tally only (fast path for auditing results)
make replay-tally BUNDLE=/path/to/eel-release-2027-08-09.tar.gz

# Compare two artifacts with diffoscope
make diff-artefact A=bundle/eel-rts-svc.tar B=rebuilt/eel-rts-svc.tar
```

## Config and environment

| Variable | Description |
|----------|-------------|
| `BUNDLE` | Path or URL to the release bundle tar |
| `IMAGES` | Space-separated list of Nix attribute paths for `verify-images` |
| `NIX_OFFLINE` | Set to `1` to force `--offline` Nix builds (requires pre-populated store) |
| `REPORT_DIR` | Output directory for diffoscope HTML/JSON and verification JSONL reports |
| `TLOG_URL` | Transparency log endpoint for attestation verification |

## Dependencies

- Nix >= 2.22 with flakes enabled
- `diffoscope` on PATH (or via `nix-shell`)
- Python 3.11+ for `hash-manifests/validate.py`
- `cosign` for signature verification
- Access to the transparency log endpoint (or an offline export for air-gapped scenarios)

## Test and verify

```bash
# Validate a hash manifest against the schema
cd reproducibility/hash-manifests
python validate.py example-manifests/bundle-v1-example.json

# Smoke test the Makefile targets
cd reproducibility/make-reproduce
make help

# Check Nix flake inputs are pinned (no floating refs)
nix flake metadata reproducibility/nix-flakes/rust | grep -E 'Inputs|Locked'
```

## Related docs

- `air-gapped-build/README.md`: offline build environment used when running `make reproduce` without internet access
- `docs/adr/0006-reproducible-builds.md`: ADR for the reproducibility approach
- `docs/audit/gate-G2.md`: gate evidence including reproducibility attestation
- `docs/plans/00-principles.md`: wave 5 (WP-5.x) covers reproducibility work packages

## Limitations

- Nix builds are reproducible for Rust and Python tiers. The Elixir/Erlang tier achieves
  near-reproducibility; known sources of non-determinism are documented in `nix-flakes/elixir/`.
- `diffoscope` comparison can take 10-20 minutes for large OCI images.
- The transparency log attestation step requires outbound HTTPS to the tlog endpoint, or
  a pre-fetched offline export. Plan for this in air-gapped environments.
