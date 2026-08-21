# integrations/card-manufacturer/contract-tests

Contract tests verifying adapter conforms to upstream API spec.

## Purpose

Ensures the adapter correctly speaks card-manufacturer's protocol.
Runs against both the mock (double) and live endpoint (when credentials available).

## Run command

```bash
cd integrations/card-manufacturer
uv run pytest -vxs contract-tests/
```

To run against live:
```bash
CARD-MANUFACTURER_BASE_URL=https://live.api contract-tests/
```

## Verification scope

- Request/response schema validation
- Authentication (headers, tokens, mTLS handshake)
- Error codes and retry behavior
- Rate limiting and backoff
- Latency SLA

## PII handling in tests

Tests strip or mask PII before logging.
See `contract-tests/conftest.py` for fixtures.
