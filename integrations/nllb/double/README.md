# integrations/nllb/double

Mock/test double for nllb.

## Purpose

Canned responses for integration testing without hitting the live upstream.
Includes chaos injection for resilience testing.

## Running the mock

```bash
cd integrations/nllb
uv run python -m double.server --port 8888
```

The mock listens on localhost:8888 by default.
Override: `NLLB_MOCK_PORT=9999`

## Chaos knobs

Environment variables enable failure modes:

```bash
NLLB_CHAOS=true uv run python -m double.server
LATENCY_MS=500 uv run python -m double.server
ERROR_RATE=0.1 uv run python -m double.server
```

## Canned responses

Responses are stored in `double/fixtures/` as JSON files.
Mapping is defined in `double/server.py`.

Example request:
```bash
curl -X POST http://localhost:8888/nllb/query \
  -H "Content-Type: application/json" \
  -d '{"query": "test"}'
```

Canned response (from fixtures).
