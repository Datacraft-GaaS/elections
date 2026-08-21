# integrations/cbk-frc

CBK/FRC aggregated SAR feed adapter for M22 finance transparency

## Wire protocol

CBK REST API; mTLS + API key; POST /sar/submit for suspicious activity reports, GET /sar/status.

### Endpoints

| Path | Description |
|---|---|
| `/cbk/sar/submit` | Submit suspicious activity report |
| `/cbk/sar/status` | Check SAR status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| accountNo | A (Restricted) |
| transactionAmount | B (Internal) |
| reportingEntity | B (Internal) |

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
| CBK_FRC_LATENCY_MS | 0 | ms delay added to every response |
| CBK_FRC_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
