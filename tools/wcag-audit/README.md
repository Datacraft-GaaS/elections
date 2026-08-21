# eel-wcag-audit

WCAG 2.2 AA/AAA accessibility compliance auditor for eel UI surfaces.

Runs axe-core against every URL listed in a manifest, produces JSON and HTML reports, and exits non-zero on any violation. M68 (referendum-explainer) requires AAA conformance; all other surfaces require AA.

## Build

```
cargo build --release -p eel-wcag-audit
```

## Usage

### Audit all URLs in the manifest

```
eel-wcag-audit audit \
  --manifest tools/wcag-audit/manifest.yaml \
  --output-dir wcag-report \
  --format both
```

Requires Node.js with `playwright` and `axe-core` installed:

```
npm install -g playwright axe-core
```

### CI / stub mode

Set `EEL_WCAG_STUB_DIR` to a directory containing pre-recorded axe JSON files (one per URL slug, or a generic `axe-pass.json` fallback):

```
EEL_WCAG_STUB_DIR=tools/wcag-audit/fixtures \
  eel-wcag-audit audit --manifest tools/wcag-audit/manifest.yaml
```

Slug derivation: strip the scheme, replace `/:.?&=#` with `_`, trim leading/trailing underscores. Example: `http://localhost:3000` becomes `localhost_3000`.

### Check a result file

```
eel-wcag-audit check --input wcag-report/wcag-report.json
eel-wcag-audit check --input wcag-report/wcag-report.json --strict
```

Exit codes: 0 = pass, 1 = violations found, 2 = browser unavailable.

### Re-render a report

```
eel-wcag-audit report --input wcag-report/wcag-report.json --format html
```

## Manifest format

```yaml
urls:
  - url: "http://localhost:3000"
    name: "public-portal"
    module: "M11"
    level: AA        # AA (default) or AAA
```

`level: AAA` is required for M68. All other modules use `level: AA`.

## Output

`--format both` (default) writes:
- `wcag-report/wcag-report.json`: machine-readable summary and per-URL results
- `wcag-report/wcag-report.html`: self-contained HTML report, theme-aware

## Tests

```
cargo test -p eel-wcag-audit
```

Tests use the fixtures in `tools/wcag-audit/fixtures/` and do not require a browser.
