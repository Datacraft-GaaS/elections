# API Client Generation

Generated clients for four languages live in `packages/api-clients/`. They are regenerated from the OpenAPI contracts in `packages/openapi-contracts/` using `just gen-clients`.

## Regenerating clients

```sh
just gen-clients
```

This runs the `gen-clients` recipe in the root `Justfile`, which:

1. Validates all `openapi.yaml` files against the OpenAPI 3.1 schema.
2. Runs `openapi-generator-cli` for each language target.
3. Writes generated code to `packages/api-clients/{rust,python,elixir,dart}/`.
4. Runs language-specific linting and type checks on the output.
5. Fails the build if any contract produces a validation error.

## Language targets

| Language | Output path | Generator | Usage |
|---|---|---|---|
| Rust | `packages/api-clients/rust/` | `rust` (reqwest) | Internal service-to-service calls, CLI tools |
| Python | `packages/api-clients/python/` | `python` (httpx async) | Django admin, LMS, analytics scripts |
| Elixir | `packages/api-clients/elixir/` | `elixir` | eel-events-ex, observer coordinator |
| Dart | `packages/api-clients/dart/` | `dart-dio` | kit-sdk-flutter (field kit and mobile apps) |

## Pinned generator version

The generator image is pinned in `tools/gen-clients/Dockerfile`. Update the pin only via an ADR or a PR reviewed by the API working group, since generator upgrades can produce breaking diffs in generated code.

## Adding a new contract

1. Create `packages/openapi-contracts/<domain>/openapi.yaml`.
2. Add the domain to `tools/gen-clients/contracts.yaml`.
3. Run `just gen-clients` and commit the generated output.
4. Add a row to [README.md](README.md) and [events.md](events.md) if the contract produces events.

## Versioning

Contracts follow semantic versioning in the `info.version` field. A minor bump is backward-compatible; a major bump requires a migration guide in `docs/guides/`. Breaking changes must not be deployed without a parallel-running v(N) endpoint for at least one release cycle.
