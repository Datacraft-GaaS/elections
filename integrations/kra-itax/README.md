# integrations/kra-itax

KRA iTax candidate tax status check adapter

## Wire protocol

KRA iTax REST API; OAuth2 token; GET /taxpayer/{pin}/status returns compliance flag.

### Endpoints

| Path | Description |
|---|---|
| `/kra/itax/status` | Check taxpayer compliance status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| pin | A (Restricted) |
| taxStatus | B (Internal) |
| fullName | A (Restricted) |

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
| KRA_ITAX_LATENCY_MS | 0 | ms delay added to every response |
| KRA_ITAX_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
