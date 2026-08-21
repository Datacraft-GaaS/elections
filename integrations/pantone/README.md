# integrations/pantone

Pantone colour verification adapter for ballot symbol colour matching

## Wire protocol

Pantone REST API; API key; GET /colour/{code} returns RGB/Lab values for Delta-E comparison.

### Endpoints

| Path | Description |
|---|---|
| `/pantone/colour/lookup` | Lookup Pantone colour values |
| `/pantone/colour/match` | Find nearest Pantone match |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| colourCode | C (Public) |
| colourName | C (Public) |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors |
| open duration | 30 s |
| retry budget | 3 retries, exponential backoff, max 10 s |

Configured via `eel-resilience-py` `CircuitBreaker`.

## Latency injection (double)

| Env var | Default | Effect |
|---|---|---|
| PANTONE_LATENCY_MS | 0 | ms delay added to every response |
| PANTONE_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
