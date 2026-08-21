# integrations/odpc-breach

ODPC 72h automated breach notification adapter per Kenya DPA section 43

## Wire protocol

ODPC REST API; API key; POST /breach/notify within 72h of detection; GET /breach/{ref}/status.

### Endpoints

| Path | Description |
|---|---|
| `/odpc/breach/notify` | Submit 72h breach notification |
| `/odpc/breach/status` | Check notification status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| dataSubjects | A (Restricted) |
| breachDescription | B (Internal) |
| contactEmail | A (Restricted) |

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
| ODPC_BREACH_LATENCY_MS | 0 | ms delay added to every response |
| ODPC_BREACH_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
