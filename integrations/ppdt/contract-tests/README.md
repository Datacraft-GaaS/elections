# integrations/ppdt/contract-tests

Contract tests verifying adapter compliance with PPDT REST API specification.

## Purpose

Ensures adapter correctly implements OAuth2 flow, request marshaling, response parsing, and error handling per PPDT spec. Tests run against mock and live (with credentials).

## Run command

```bash
# Against mock (default, no credentials needed)
uv run pytest -vxs tests/ci/

# Against live endpoint
PPDT_BASE_URL=https://api.ppdt.go.ke uv run pytest -vxs tests/ci/
```

## What is verified

- OAuth2 token acquisition and refresh
- Request schema validation (party registration, lookups)
- Response schema and data types
- HTTP status codes (200, 400, 401, 429, 5xx)
- Retry logic and circuit breaker signals
- Rate limit headers (X-RateLimit-Remaining, Retry-After)
- Request/response timestamps and cache directives
- Error messages and exception mapping
- PII masking in logs

Tests use real Pydantic validation (no mocks on models).
