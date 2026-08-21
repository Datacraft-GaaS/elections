# eel Python API Client

Generated Python clients for all eel OpenAPI contexts.

## Regenerate

From the repo root:

```sh
just gen-clients
```

Or from this directory:

```sh
./generate.sh --regen
```

## What the generated code contains

For each OpenAPI context (e.g. `elections`, `results`), a Python package is generated under `generated/<ctx>/` using the `python` openapi-generator template. Each package includes:

- Typed model classes via dataclasses or Pydantic
- An API class with methods for each endpoint
- A configuration class for base URL and auth

Package namespace: `eel_api_client_<ctx>`

## Usage

Install the generated package in development mode:

```sh
pip install -e packages/api-clients/py/generated/elections
```

Then use it:

```python
from eel_api_client_elections import ApiClient, Configuration, DefaultApi

cfg = Configuration(host="https://api.eel.example.com")
with ApiClient(cfg) as client:
    api = DefaultApi(client)
    elections = api.list_elections()
```

## Notes

- The `generated/` directory is gitignored. Run `./generate.sh --regen` after pulling new OpenAPI specs.
- Requires `openapi-generator-cli` (via npm, Homebrew, or Docker). `ruff format` is applied post-generation if available.
- Package name pattern: `eel_api_client_<ctx>`
