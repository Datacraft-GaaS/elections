# integrations/mck/contract-tests

Contract tests verifying MCK adapter compliance with API specification.

## Purpose

Ensures adapter correctly implements API-key auth, request marshaling, and response handling per MCK spec. Tests run against mock and live (with credentials).

## Run command

```bash
# Against mock (default, no credentials needed)
uv run pytest -vxs tests/ci/

# Against live
MCK_BASE_URL=https://api.mck.or.ke uv run pytest -vxs tests/ci/
```

## What is verified

- API-key authentication and headers
- Request schema validation (accreditation, credential lookups)
- Response schema and data types
- HTTP status codes (200, 400, 401, 429, 5xx)
- Retry logic and circuit breaker signals
- Rate limit headers
- PII masking in logs
- Accreditation status enum values

Tests use real Pydantic validation.
