# integrations/knads

KNADS national archival and documentation service adapter

## Wire protocol

KNADS REST API; API key; endpoints for document submission, retrieval, and archival confirmation.

### Endpoints

| Path | Description |
|---|---|
| `/knads/document/submit` | Submit document for archival |
| `/knads/document/retrieve` | Retrieve archived document |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| documentContent | B (Internal) |
| archivalRef | B (Internal) |

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
| KNADS_LATENCY_MS | 0 | ms delay added to every response |
| KNADS_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
