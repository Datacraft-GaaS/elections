# integrations/cbk-frc/adapter

Adapter implementation for cbk-frc.

## Purpose

Isolates authentication, marshaling, and error handling for the cbk-frc integration.
Exposes a clean Python interface via the `adapt()` function.

## Layout

- `service.py` - Main service class with request/response handling
- `views.py` - Pydantic models (requests, responses, errors)
- `__init__.py` - Public API export
- `conftest.py` - Pytest fixtures for testing

## Key classes

See `views.py` for exact schemas.

Key methods in `service.py`:
- `__init__(config)` - Initialize with credentials/endpoints
- `call(operation, payload)` - Execute a single operation
- `batch(operations)` - Execute multiple operations in parallel (where supported)

## Environment variables

All required for operation. Obtain from infrastructure/ops.

## Build and test

```bash
cd integrations/cbk-frc
uv run pytest -vxs adapter/tests/
uv run pyright adapter/
```

For mTLS setup, ensure certificates are in PEM format and readable.

## Authentication setup

See parent README.md for auth scheme details.
Credentials stored in env vars or mounted secrets (see `.env.example`).
