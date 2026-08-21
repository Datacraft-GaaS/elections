# integrations/nssf

NSSF social security contributions adapter

## Wire protocol

NSSF REST API; API key in X-API-Key header; endpoints for contribution lookup and remittance.

### Endpoints

| Path | Description |
|---|---|
| `/nssf/member/lookup` | Lookup NSSF member |
| `/nssf/contribution/remit` | Submit contribution |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| nationalId | A (Restricted) |
| memberNo | A (Restricted) |
| contributionAmount | B (Internal) |

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
| NSSF_LATENCY_MS | 0 | ms delay added to every response |
| NSSF_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
