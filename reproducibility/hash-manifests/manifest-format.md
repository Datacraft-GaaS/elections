# Hash Manifest Format

Version: 1

## Purpose

A hash manifest is a signed, versioned record of the sha256 digests, build provenance, and SBOM references for every artefact produced by an eel build wave. It is the authoritative input to `make reproduce`.

## Schema

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

## Field definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | string | yes | Must be `"1"` for this revision. |
| `bundle_id` | string | yes | Unique bundle identifier. |
| `created_at` | string (ISO 8601) | yes | UTC timestamp of manifest creation. |
| `tally_root` | string (64-char hex) | yes | BLAKE3 Merkle root derived by replaying `events.jsonl`. |
| `artefacts[].name` | string | yes | Human-readable artefact name; unique within the manifest. |
| `artefacts[].kind` | string enum | yes | One of: `image`, `binary`, `library`, `policy`, `schema`. |
| `artefacts[].sha256` | string (64-char hex) | yes | SHA-256 of the artefact content. |
| `artefacts[].source_ref` | string (40-char hex) | yes | Git commit SHA at which the artefact was built. |
| `artefacts[].build_recipe` | string | no | Nix attribute path used to reproduce the artefact (e.g. `packages.x86_64-linux.eel-vprs`). |
| `artefacts[].sbom_ref` | string | no | Bundle-relative path to the CycloneDX or SPDX SBOM JSON. |
| `artefacts[].cosign_sig_ref` | string | no | Bundle-relative path to the cosign detached signature. |

## Validation

Run `validate.py` to check structural conformance and cross-check sha256 values against a bundle contents index:

```sh
uv run python validate.py manifest.json bundle-index.json
```

## Versioning

Increment `schema_version` (as a string integer) for backward-incompatible changes. `validate.py` rejects unknown versions.
