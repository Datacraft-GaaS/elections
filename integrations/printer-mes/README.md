# integrations/printer-mes

Ballot printer MES (manufacturing execution system) double adapter

## Wire protocol

MES REST API; API key; POST /print/order for ballot print job, GET /print/order/{id}/status.

### Endpoints

| Path | Description |
|---|---|
| `/printer/order/submit` | Submit print order |
| `/printer/order/status` | Check print order status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| printOrderRef | B (Internal) |
| ballotQty | B (Internal) |

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
| PRINTER_MES_LATENCY_MS | 0 | ms delay added to every response |
| PRINTER_MES_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
