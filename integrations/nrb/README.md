# integrations/nrb

NRB National Registration Bureau ID issuance status adapter

## Wire protocol

NRB REST API; API key; GET /id/{application_ref}/status returns issuance state.

### Endpoints

| Path | Description |
|---|---|
| `/nrb/id/status` | Check ID issuance status |
| `/nrb/id/application` | Submit ID application lookup |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| applicationRef | A (Restricted) |
| applicantName | A (Restricted) |

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
| NRB_LATENCY_MS | 0 | ms delay added to every response |
| NRB_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
