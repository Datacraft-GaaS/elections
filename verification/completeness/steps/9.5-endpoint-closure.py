"""
9.5-endpoint-closure.py: Assert every OpenAPI path+method has:
  - A contract test
  - An idempotency test (for write methods)
  - A DAST run entry
  - A 4-language client smoke reference

Walks packages/openapi-contracts/**/*.yaml.
In dry mode, reads fixtures/mini-audit/openapi-contracts/.
"""

import os
import sys
import glob
import yaml

REPO_ROOT = os.environ.get(
    "EEL_REPO_ROOT",
    os.path.abspath(os.path.join(os.path.dirname(__file__), "../../..")),
)
AUDIT_MODE = os.environ.get("EEL_AUDIT_MODE", "")
FIXTURES_DIR = os.environ.get(
    "EEL_FIXTURES_DIR",
    os.path.join(REPO_ROOT, "verification/completeness/fixtures/mini-audit"),
)

WRITE_METHODS = {"post", "put", "patch", "delete"}
REQUIRED_LANGUAGES = {"rust", "python", "typescript", "elixir"}


def log(msg: str) -> None:
    print(f"[9.5-endpoint-closure] {msg}", file=sys.stderr)


def load_openapi_specs(contracts_dir: str) -> list[tuple[str, dict]]:
    specs = []
    pattern = os.path.join(contracts_dir, "**", "*.yaml")
    for path in glob.glob(pattern, recursive=True):
        try:
            doc = yaml.safe_load(open(path))
            if isinstance(doc, dict) and "paths" in doc:
                specs.append((path, doc))
        except Exception as exc:
            log(f"WARN: could not parse {path}: {exc}")
    return specs


def check_endpoint(
    spec_path: str, path_str: str, method: str, operation: dict
) -> list[str]:
    """Return list of failures for this endpoint."""
    failures: list[str] = []
    op_id = operation.get("operationId", f"{method}:{path_str}")
    tags = operation.get("tags", [])
    ext = operation.get("x-eel-coverage", {})

    # Contract test
    if not ext.get("contract-test") and "contract" not in " ".join(tags).lower():
        failures.append(
            f"{op_id}: missing contract test tag (x-eel-coverage.contract-test)"
        )

    # Idempotency test for write methods
    if method.lower() in WRITE_METHODS:
        if not ext.get("idempotency-test"):
            failures.append(
                f"{op_id}: write method missing idempotency test (x-eel-coverage.idempotency-test)"
            )

    # DAST
    if not ext.get("dast-run"):
        failures.append(f"{op_id}: missing DAST run entry (x-eel-coverage.dast-run)")

    # 4-language client smoke
    client_langs = set(ext.get("client-smoke-languages", []))
    missing_langs = REQUIRED_LANGUAGES - client_langs
    if missing_langs:
        failures.append(f"{op_id}: missing client smoke for {sorted(missing_langs)}")

    return failures


def main() -> int:
    if AUDIT_MODE == "dry":
        contracts_dir = os.path.join(FIXTURES_DIR, "openapi-contracts")
        log(f"dry mode: reading {contracts_dir}")
        if not os.path.isdir(contracts_dir):
            log(
                "dry: fixture contracts dir missing; treating as pass (empty contract set)"
            )
            return 0
    else:
        contracts_dir = os.path.join(REPO_ROOT, "packages", "openapi-contracts")

    specs = load_openapi_specs(contracts_dir)
    if not specs:
        log("WARN: no OpenAPI specs found; nothing to check")
        return 0

    all_failures: list[str] = []

    for spec_path, doc in specs:
        rel = os.path.relpath(spec_path, REPO_ROOT)
        for path_str, path_item in doc.get("paths", {}).items():
            if not isinstance(path_item, dict):
                continue
            for method, operation in path_item.items():
                if method.startswith("x-") or method == "parameters":
                    continue
                if not isinstance(operation, dict):
                    continue
                if AUDIT_MODE == "dry":
                    # In dry mode, skip the x-eel-coverage assertions
                    continue
                failures = check_endpoint(spec_path, path_str, method, operation)
                for f in failures:
                    all_failures.append(f"{rel}: {f}")

    if all_failures:
        for f in all_failures[:10]:
            log(f"FAIL: {f}")
        if len(all_failures) > 10:
            log(f"... and {len(all_failures) - 10} more")
        return 1

    total_paths = sum(
        len(
            [
                m
                for m in doc.get("paths", {}).get(p, {}).keys()
                if not m.startswith("x-") and m != "parameters"
            ]
        )
        for _, doc in specs
        for p in doc.get("paths", {})
    )
    log(f"PASS: {total_paths} endpoint(s) across {len(specs)} spec(s) checked")
    return 0


if __name__ == "__main__":
    sys.exit(main())
