# integrations/mck/double

Test double for MCK API-key REST API.

## Purpose

Canned accreditation and credential responses for local testing without MCK backend. Supports chaos injection for circuit breaker validation.

## Run command

```bash
uv run python -m double.server --port 8888
# Or: MCK_CHAOS=timeout uv run python -m double.server
```

Listens on `localhost:8888/`.

## Chaos knobs

```bash
MCK_CHAOS=timeout uv run python -m double.server  # 504 after 15s
MCK_CHAOS=error uv run python -m double.server    # 503 Service Unavailable
MCK_CHAOS=slow uv run python -m double.server     # add 5s latency
LATENCY_MS=2000 ERROR_RATE=0.2 uv run python -m double.server
```

## Canned scenarios

- `scenario/accreditation-approved.json` - Journalist accredited
- `scenario/accreditation-pending.json` - Under review
- `scenario/credential-invalid.json` - Expired or cancelled
- `scenario/not-found.json` - No match in register
- `scenario/rate-limit.json` - 429 Too Many Requests
