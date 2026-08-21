# reproducibility/hash-manifests

Signed, versioned records of sha256 digests, build provenance, and SBOM references for
every artifact produced by an eel build wave. Hash manifests are the authoritative input
to `make reproduce` and the root of the evidentiary chain for compliance audits.

## Purpose

A hash manifest answers: "what was built, from which commit, with which recipe, and does
it match what is deployed?" Every release bundle includes one signed manifest. Auditors
compare the manifest against transparency log entries and against independently rebuilt
artifacts.

## Schema (version 1)

```json
{
  "schema_version": "1",
  "bundle_id": "<string>",
  "created_at": "<ISO 8601 UTC>",
  "tally_root": "<64-char BLAKE3 hex>",
  "artefacts": [
    {
      "name": "<string>",
      "kind": "<image|binary|library|policy|schema>",
      "sha256": "<64-char hex>",
      "source_ref": "<40-char git commit SHA>",
      "build_recipe": "<nix attr path>",
      "sbom_ref": "<relative path to SBOM JSON in bundle>",
      "cosign_sig_ref": "<relative path to cosign .sig in bundle>"
    }
  ]
}
```

See `manifest-format.md` for the full field specification including required/optional
status, validation rules, and versioning policy.

## Contents

| File/Directory | Description |
|----------------|-------------|
| `manifest-format.md` | Full schema specification with field definitions and examples |
| `schemas/manifest-v1.schema.json` | JSON Schema for manifest validation |
| `validate.py` | CLI validator: schema check, signature verification, bundle cross-reference |
| `example-manifests/` | Sample manifests for testing and documentation |
| `pyproject.toml`, `uv.lock` | Python tooling for `validate.py` |
| `tests/` | pytest tests for the validator |

## Usage

```bash
# Validate a manifest against the schema
cd reproducibility/hash-manifests
uv run python validate.py /path/to/bundle/manifest.json

# Validate and cross-reference against a bundle
uv run python validate.py \
  --bundle /path/to/eel-release.tar.gz \
  /path/to/bundle/manifest.json

# Run validator tests
uv run pytest -vxs tests/ci
```

## Config and environment

The validator reads the following optional environment variables:

| Variable | Description |
|----------|-------------|
| `COSIGN_PUBLIC_KEY` | Path to the cosign public key for manifest signature verification |
| `TLOG_URL` | Transparency log endpoint for online attestation verification |
| `TLOG_OFFLINE_EXPORT` | Path to an offline tlog export JSON (air-gapped use) |

## Dependencies

- Python 3.11+
- `uv` for dependency management
- `cosign` CLI for signature verification (optional; validator skips sig check if absent)
- `jsonschema` Python package (managed via `uv`)

## Test and verify

```bash
cd reproducibility/hash-manifests
uv run pytest -vxs tests/ci

# Validate the example manifest (should pass)
uv run python validate.py example-manifests/bundle-v1-example.json
# Expected: "Manifest valid. 0 errors."

# Intentionally corrupt a field and confirm rejection
python -c "
import json, sys
m = json.load(open('example-manifests/bundle-v1-example.json'))
m['tally_root'] = 'aabbccdd'  # wrong length
print(json.dumps(m))
" | uv run python validate.py /dev/stdin
# Expected: validation error on tally_root
```

## Related docs

- `reproducibility/README.md`: full workflow
- `reproducibility/make-reproduce/README.md`: pipeline that consumes manifests
- `docs/adr/0006-reproducible-builds.md`: design decisions

## Limitations

- Schema version 1 is the only supported version. A new ADR is required to bump to v2.
- Signature verification requires the cosign public key matching the key used at build
  time. Rotate the key only via the key management process documented in
  `docs/runbooks/key-rotation.md`.
- The validator does not check that all bundle files listed in `artefacts[].sbom_ref`
  exist on disk when running without `--bundle`; pass `--bundle` for a complete check.
