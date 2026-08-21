# integrations/ppdt/double

Test double (mock server) for PPDT OAuth2 REST API.

## Purpose

Canned OAuth2 and API responses for local testing without live PPDT backend. Supports chaos injection for circuit breaker and retry logic validation.

## Run command

```bash
cd integrations/ppdt
uv run python -m double.server --port 8888
# Or export to override defaults:
PPDT_BASE_URL=http://localhost:8888 PPDT_CHAOS=true uv run python -m double.server
```

Mock listens on `localhost:8888/` (override: `PPDT_MOCK_PORT` env var).

## Chaos knobs

Set via environment:

```bash
# Timeout (504 after 15s)
PPDT_CHAOS=timeout uv run python -m double.server

# Error (503 Service Unavailable)
PPDT_CHAOS=error uv run python -m double.server

# Slow (add 5s latency)
PPDT_CHAOS=slow uv run python -m double.server

# Combined
LATENCY_MS=2000 ERROR_RATE=0.2 TIMEOUT_SECS=20 uv run python -m double.server
```

## Canned scenarios

- `scenario/register-success.json` - Party registration approved
- `scenario/register-pending.json` - Registration under review
- `scenario/lookup-found.json` - Party details lookup success
- `scenario/lookup-not-found.json` - Party not in register
- `scenario/auth-error.json` - OAuth2 token rejection
- `scenario/rate-limit.json` - 429 Too Many Requests

Request path determines response; see `double/server.py` for mapping.
