# integrations/kebs-warranty

KEBS warranty and standards certification verification adapter

## Wire protocol

KEBS REST API; API key; GET /product/{serial}/certification returns standards compliance.

### Endpoints

| Path | Description |
|---|---|
| `/kebs/certification/verify` | Verify product certification |
| `/kebs/warranty/status` | Check warranty status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| serialNo | B (Internal) |
| productCode | C (Public) |

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
| KEBS_WARRANTY_LATENCY_MS | 0 | ms delay added to every response |
| KEBS_WARRANTY_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
