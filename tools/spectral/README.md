# tools/spectral/

OpenAPI Spectral ruleset for the eel IEMS. Enforces ADR-0011 (API contracts: tiers, RFC 9457 error bodies, JSON+CBOR content types, signing headers, Idempotency-Key) across all OpenAPI specs in `packages/openapi-contracts/`.

## Contents

| File | Description |
|---|---|
| `functions/eel-has-content-types.js` | Custom function: verifies `application/json` and `application/cbor` are both declared |
| `functions/eel-has-data-class.js` | Custom function: verifies `x-eel-data-class` extension is set on every response schema |

The Spectral ruleset root (`.spectral.yaml`) lives at the repository root and references this directory.

## Prerequisites

```bash
npm install -g @stoplight/spectral-cli
```

## Usage

```bash
# Lint all OpenAPI contracts:
spectral lint packages/openapi-contracts/**/*.yaml

# Lint a single contract:
spectral lint packages/openapi-contracts/rts/results-api.yaml

# JSON output for CI:
spectral lint packages/openapi-contracts/**/*.yaml --format json --output spectral-report.json

# Fail only on errors (ignore warnings):
spectral lint packages/openapi-contracts/**/*.yaml --fail-severity error
```

## Rules enforced

| Rule | Severity | ADR |
|---|---|---|
| `eel-has-content-types` | error | ADR-0011: both JSON and CBOR required |
| `eel-has-data-class` | error | ADR-0011: data classification on every response |
| `eel-idempotency-key-on-post` | error | ADR-0011: POST/PUT must declare `Idempotency-Key` header |
| `eel-rfc9457-error-body` | error | ADR-0011: error responses must use RFC 9457 `application/problem+json` |
| `eel-no-nullable-required` | warn | API hygiene: no `required` field marked `nullable` |
| `eel-versioned-paths` | warn | ADR-0011: paths must carry `/v{n}/` segment |

## CI integration

```yaml
# .woodpecker.yml (example step):
- name: spectral-lint
  image: stoplight/spectral:6
  commands:
    - spectral lint packages/openapi-contracts/**/*.yaml --fail-severity error
```

Clients are regenerated after a passing lint via `just gen-clients`.

## Exit codes

Spectral standard: `0` = pass, `1` = lint errors present.

## Related docs

- `docs/adr/0011-api-contracts.md`: ADR-0011 (API contract conventions)
- `packages/openapi-contracts/README.md`: OpenAPI contract directory structure
- `justfile`: `just gen-clients` regenerates SDK clients after contract changes
