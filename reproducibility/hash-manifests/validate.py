#!/usr/bin/env python3
"""validate.py -- Validate a hash manifest against the JSON Schema and
optionally cross-check sha256 values against a bundle contents index.

Usage:
    uv run python validate.py <manifest.json> [<bundle-index.json>]

Bundle index format (optional):
    {
      "contents": {
        "<name>": "<64-char sha256 hex>",
        ...
      }
    }

Exit codes: 0 = all checks pass, 1 = validation error.
"""

import json
import sys
from pathlib import Path

try:
    import jsonschema
    from jsonschema import validate as jschema_validate, ValidationError
except ImportError:
    print(
        "ERROR: jsonschema not installed. Run: uv pip install jsonschema",
        file=sys.stderr,
    )
    sys.exit(1)

SCHEMA_PATH = Path(__file__).parent / "schemas" / "hash-manifest.schema.json"
SUPPORTED_VERSIONS = {"1"}


def load_json(path: str) -> dict:
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        print(f"ERROR: cannot load {path}: {e}", file=sys.stderr)
        sys.exit(1)


def validate_manifest(manifest: dict) -> list[str]:
    """Return list of error strings; empty means valid."""
    errors: list[str] = []

    # Schema version check first (fast path before full JSON Schema).
    sv = manifest.get("schema_version")
    if sv not in SUPPORTED_VERSIONS:
        errors.append(f"Unsupported schema_version: {sv!r}")
        return errors

    schema = load_json(str(SCHEMA_PATH))
    try:
        jschema_validate(instance=manifest, schema=schema)
    except ValidationError as e:
        errors.append(
            f"JSON Schema validation failed: {e.message} (path: {list(e.absolute_path)})"
        )

    return errors


def cross_check(manifest: dict, index: dict) -> list[str]:
    """Compare manifest sha256 values against bundle-index contents."""
    errors: list[str] = []
    contents: dict[str, str] = index.get("contents", {})

    for art in manifest.get("artefacts", []):
        name = art.get("name", "<unknown>")
        declared = art.get("sha256", "")
        if name in contents:
            actual = contents[name]
            if actual != declared:
                errors.append(
                    f"sha256 mismatch for {name!r}: "
                    f"manifest={declared!r} index={actual!r}"
                )
        else:
            errors.append(f"Artefact {name!r} not found in bundle index")

    return errors


def main() -> None:
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    manifest_path = sys.argv[1]
    index_path = sys.argv[2] if len(sys.argv) > 2 else None

    manifest = load_json(manifest_path)
    errors = validate_manifest(manifest)

    if not errors and index_path:
        index = load_json(index_path)
        errors.extend(cross_check(manifest, index))

    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        sys.exit(1)

    artefact_count = len(manifest.get("artefacts", []))
    print(
        f"OK: manifest {manifest_path!r} valid "
        f"(schema_version={manifest['schema_version']}, "
        f"{artefact_count} artefact(s))"
    )


if __name__ == "__main__":
    main()
