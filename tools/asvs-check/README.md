# eel-asvs-check

CLI tool that scans the eel repo for test trace tags and maps them to ASVS 4.0
L2 (routine) and L3 (election-critical) requirements. Produces a compliance
report showing which requirements have test coverage and which are gaps.

## Installation

Build from the workspace root:

```
cargo build --release -p eel-asvs-check
```

The binary is at `target/release/eel-asvs-check`.

## ASVS manifest

The tool reads `tools/asvs-check/asvs-manifest.yaml` by default (relative to
the repository root). Pass `--manifest /path/to/file.yaml` to override.

Each entry in the manifest has:

- `id`: ASVS requirement ID (e.g. `V6.2.3`)
- `level`: `L2` or `L3`
- `description`: human-readable text
- `mapped_tags`: list of repo trace tags (e.g. `EC-DAT-01`) that count as
  coverage for this requirement

## Test tag conventions

The scanner recognises tags in all repo languages:

| Language | Syntax |
|----------|--------|
| Rust | `#[trace("ASVS-V6.2.3", "EC-DAT-01")]` |
| Python | `@pytest.mark.trace("ASVS-V6.2.3")` |
| Elixir | `@tag trace: ["ASVS-V6.2.3"]` |
| Dart/Flutter | `testWidgets('name [ASVS-V6.2.3]', ...)` |
| JS/TS (Playwright) | `test('name @ASVS-V6.2.3', ...)` |

A requirement is covered if any test in the repo carries:
- a tag matching `ASVS-<requirement-id>` (e.g. `ASVS-V6.2.3`), or
- a tag from the requirement's `mapped_tags` list.

## Subcommands

### scan

Walk the repo and print per-requirement coverage status:

```
eel-asvs-check scan
eel-asvs-check --root /path/to/eel scan
```

### report

Emit a full compliance report:

```
eel-asvs-check report --format md
eel-asvs-check report --format json --output asvs-report.json
eel-asvs-check report --format html --output asvs-report.html
```

Formats: `md` (default), `json`, `html`.

### check

Exit non-zero if any L3 requirement has no test coverage (CI gate):

```
eel-asvs-check check
```

With `--strict`, also fail on L2 gaps:

```
eel-asvs-check check --strict
```

## Exit codes

- `0`: all requirements covered (or no gaps at the checked level)
- `1`: gaps found (check subcommand only)

## CI integration

Add to the CI pipeline after `cargo test`:

```yaml
- name: ASVS coverage check
  run: eel-asvs-check --root . check
```

For a full report artifact:

```yaml
- name: ASVS report
  run: eel-asvs-check report --format html --output docs/audit/asvs-report.html
```
