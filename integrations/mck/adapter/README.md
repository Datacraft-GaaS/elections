# integrations/mck/adapter

Adapter for Media Council of Kenya API-key authenticated journalist accreditation integration.

## Purpose

Encapsulates API-key auth, request marshaling, response parsing, and error handling for MCK accreditation lookups. Exposes clean async Python interface.

## Layout

- `service.py` - `MCKAdapter` service with API-key handling, accreditation methods
- `views.py` - Pydantic v2 models (requests, responses, validation rules)
- `errors.py` - MCK exception hierarchy
- `config.py` - Configuration loader (env vars, validation)
- `tests/` - Unit tests (fixtures, mocks)

## Key classes

- `MCKAdapter` - Async service, API-key per-request, circuit breaker-aware
- `AccreditationRequest`, `AccreditationResponse` - Accreditation schema
- `CredentialVerification` - Journalist credential response
- `MCKError`, `MCKAuthError`, `MCKNotFoundError` - Exception types

Validation: `extra='forbid'`, `validate_by_name=True` per project standards.

## Environment variables

| Variable | Required | Purpose |
|---|---|---|
| MCK_BASE_URL | yes | API endpoint |
| MCK_API_KEY | yes | API key for authentication |
| MCK_TIMEOUT_SECS | no | Request timeout (default: 10) |

## Build and test

```bash
# Install
uv pip install -e .

# Type check
uv run pyright adapter/

# Test (no mocks; uses pytest-httpserver for REST)
uv run pytest -vxs tests/ci/test_mck_adapter.py

# Lint
uv run ruff check adapter/
```

All tests run against real Pydantic validation, no mocking of models.
