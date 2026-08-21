# eel-completeness

Stub scanner, traceability coverage mapper, and contract closure verifier for the eel repo.
Used by Wave 0 CI gate (WP-0.5) and the Wave 6 completeness audit (WP-6.7, section 9.1).

## Build

```
cargo build --release -p eel-completeness
```

## Commands

### scan

Walk the repository and emit stub/skip/todo findings.

```
eel-completeness scan
eel-completeness scan --strict
eel-completeness scan --strict --format json
eel-completeness --allowlist .completeness-allowlist.yaml scan --strict
```

Flags:

| Flag | Description |
|---|---|
| `--strict` | Fail on any finding, including allowlisted ones. Used by Wave 6 gate 9.1. |
| `--fail-on-new` | Fail only on findings not in the allowlist. Used as PR gate. |
| `--format human` | Human-readable output (default). |
| `--format json` | Machine-readable JSON output. Required by `verification/completeness/steps/9.1-stub-scan.sh`. |
| `--format junit` | JUnit XML for CI integration. |
| `--allowlist <path>` | Path to `.completeness-allowlist.yaml` (default: repo root). |
| `--root <path>` | Repository root to scan (default: current directory). |

JSON output format:

```json
{
  "findings": [
    {
      "file": "services/foo/src/bar.rs",
      "line": 42,
      "kind": "todo",
      "text": "todo!(\"implement me\")",
      "allowlisted": false
    }
  ],
  "total": 1,
  "allowlisted": 0,
  "new": 1
}
```

Exit codes: 0 = pass, 1 = findings that fail the configured gate, 2 = internal error.

### trace

Build a traceability coverage map from test trace annotations.

```
eel-completeness trace --traceability docs/plans/TRACEABILITY.yaml --strict
eel-completeness trace --format json
```

### contract-closure

Verify event schema producer/consumer coverage.

```
eel-completeness contract-closure --format json
```

## Allowlist format

`.completeness-allowlist.yaml` at the repo root:

```yaml
allowlist:
  - file: "services/foo/src/clickhouse.rs"
    kind: todo
    reason: "ClickHouse migration deferred (ADR-0021)"
    adr: "ADR-0021"
    expires: "2026-12-31"
```

Any finding matching a non-expired allowlist entry is suppressed from `--fail-on-new` but
still reported by `--strict`.

## Integration with WP-6.7

`verification/completeness/steps/9.1-stub-scan.sh` invokes:

```
eel-completeness --root <repo> --allowlist .completeness-allowlist.yaml scan --strict --format json
```

and asserts exit 0 (zero non-allowlisted findings).

## Tests

```
cargo test -p eel-completeness
```

Fixtures in `tools/completeness/fixtures/` do not require a live repo.
