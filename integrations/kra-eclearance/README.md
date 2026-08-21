# integrations/kra-eclearance

KRA eClearance customs duty and clearance adapter

## Wire protocol

KRA eClearance REST API; OAuth2; POST /entry/submit for customs entry, GET /entry/{ref}/status.

### Endpoints

| Path | Description |
|---|---|
| `/kra/eclearance/entry/submit` | Submit customs entry |
| `/kra/eclearance/entry/status` | Check entry status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| importerPin | A (Restricted) |
| entryRef | B (Internal) |
| dutyAmount | B (Internal) |

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
| KRA_ECLEARANCE_LATENCY_MS | 0 | ms delay added to every response |
| KRA_ECLEARANCE_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
