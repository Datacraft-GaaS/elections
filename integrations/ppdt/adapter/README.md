# integrations/ppdt/adapter

Adapter for Party Petitioning Database OAuth2-authenticated REST integration.

## Purpose

Encapsulates OAuth2 token lifecycle, request marshaling, response parsing, and error handling for PPDT REST API calls. Exposes clean async Python interface.

## Layout

- `service.py` - `PPDTAdapter` service with OAuth2 token refresh, API methods
- `views.py` - Pydantic v2 models (requests, responses, validation rules)
- `errors.py` - PPDT exception hierarchy
- `config.py` - Configuration loader (env vars, validation)
- `tests/` - Unit tests (fixtures, mocks)

## Key classes

- `PPDTAdapter` - Async service, auto token refresh, circuit breaker-aware
- `PartyRegistrationRequest` - Party registration payload (Annotated validators)
- `PartyLookupResponse` - Party details response with caching metadata
- `PPDTError`, `PPDTAuthError`, `PPDTRateLimitError` - Exception types

Validation: `extra='forbid'`, `validate_by_name=True` per project standards.

## Environment variables

| Variable | Required | Purpose |
|---|---|---|
| PPDT_BASE_URL | yes | API endpoint |
| PPDT_CLIENT_ID | yes | OAuth2 client ID |
| PPDT_CLIENT_SECRET | yes | OAuth2 client secret |
| PPDT_TOKEN_URL | yes | OAuth2 token endpoint |
| PPDT_TIMEOUT_SECS | no | Request timeout (default: 10) |

## Build and test

```bash
# Install
uv pip install -e .

# Type check
uv run pyright adapter/

# Test (no mocks; uses pytest-httpserver for REST)
uv run pytest -vxs tests/ci/test_ppdt_adapter.py

# Lint
uv run ruff check adapter/
```

All tests: run against real Pydantic validation, no mocking of models.

