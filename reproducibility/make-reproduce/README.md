# reproducibility/make-reproduce

Makefile-driven pipeline for reproducing and verifying eel release artifacts. This is the
primary entry point for auditors and the Commission's technical team to confirm that deployed
binaries match reviewed source.

## Purpose

`make reproduce` orchestrates the full verification pipeline:

1. Fetch and unpack the release bundle.
2. Verify hash manifest signature against the transparency log.
3. Rebuild every listed artifact from source via Nix.
4. Compare rebuilt artifacts against bundle artifacts using `diffoscope`.
5. Replay the event log to re-derive the tally Merkle root.
6. Write a machine-readable report to `.repro-reports/`.

Each step is also available as an individual Make target for partial runs.

## Makefile targets

| Target | Description |
|--------|-------------|
| `reproduce BUNDLE=<path>` | Full pipeline: fetch, verify, rebuild, diff, replay-tally |
| `verify-images IMAGES=<list>` | Rebuild named Nix attrs; compare sha256 against bundle |
| `replay-tally BUNDLE=<path>` | Replay `events.jsonl`; compare derived Merkle root |
| `diff-artefact A=<p> B=<p>` | `diffoscope` wrapper; outputs HTML + JSON report |
| `clean-repro` | Remove `.repro-reports/` workspace |
| `help` | Print target list with descriptions |

## Contents

| File | Description |
|------|-------------|
| `Makefile` | Top-level pipeline definition |
| `scripts/reproduce.sh` | Full pipeline shell script called by `make reproduce` |
| `scripts/replay-tally.sh` | Event log replay script |
| `docker/` | Dockerfile for the reproducer container image |
| `docker-compose.yml` | Local reproducer environment with Nix and diffoscope pre-installed |
| `fixtures/` | Small test bundle fixtures for CI smoke tests |
| `tests/` | pytest-based integration tests for the pipeline scripts |

## Usage

```bash
# Full pipeline from a local bundle
cd reproducibility/make-reproduce
make reproduce BUNDLE=/path/to/eel-release-bundle.tar.gz

# Partial: rebuild and diff one image
make verify-images IMAGES="eel-rts-svc"
make diff-artefact \
  A=.repro-reports/bundle-eel-rts-svc.tar \
  B=.repro-reports/rebuilt-eel-rts-svc.tar

# Partial: tally replay only (fast, useful during results audit)
make replay-tally BUNDLE=/path/to/eel-release-bundle.tar.gz

# Docker-based reproducer (no local Nix required)
docker compose run reproducer make reproduce BUNDLE=/bundles/eel-release.tar.gz

# Run CI smoke tests against fixtures
uv run pytest -vxs tests/ci
```

## Config and environment

| Variable | Default | Description |
|----------|---------|-------------|
| `BUNDLE` | required | Path or HTTPS URL to release bundle tar |
| `IMAGES` | required for verify-images | Space-separated Nix attr paths |
| `A`, `B` | required for diff-artefact | Paths to compare |
| `NIX_OFFLINE` | `0` | Set `1` to pass `--offline` to nix |
| `REPRO_DIR` | `$(CURDIR)` | Root of the make-reproduce directory |
| `REPORT_DIR` | `$(REPRO_DIR)/.repro-reports` | Output directory |

## Dependencies

- GNU make >= 4.3
- Nix >= 2.22 with flakes enabled
- `diffoscope` on PATH
- `jq` for JSON processing in scripts
- Python 3.11+ for `hash-manifests/validate.py` (called by `reproduce.sh`)
- `cosign` for manifest signature verification

## Test and verify

```bash
# Smoke test against fixtures (runs in CI)
cd reproducibility/make-reproduce
uv run pytest -vxs tests/ci

# Test replay-tally against a known-good fixture
make replay-tally BUNDLE=fixtures/bundle-v1-test.tar.gz
cat .repro-reports/replay-tally.json | jq .tally_match
# Expected: true
```

## Related docs

- `reproducibility/README.md`: workflow overview
- `reproducibility/hash-manifests/README.md`: manifest schema
- `reproducibility/nix-flakes/README.md`: Nix build fragments
- `air-gapped-build/README.md`: offline build environment

## Limitations

- `reproduce.sh` does not retry on network failures; pre-fetch the bundle before running.
- `diffoscope` comparison on large OCI images can use 4GB+ of RAM.
- The `replay-tally` step requires the `events.jsonl` file to be inside the bundle; partial
  bundles produced for intermediate waves may omit it.
