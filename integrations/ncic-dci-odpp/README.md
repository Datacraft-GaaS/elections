# integrations/ncic-dci-odpp

NCIC/DCI/ODPP referral and criminal record check adapter

## Wire protocol

Internal REST gateway; mTLS; POST /ncic/check for criminal history, POST /odpp/referral for case handoff.

### Endpoints

| Path | Description |
|---|---|
| `/ncic/check` | Criminal history check |
| `/dci/referral/submit` | Submit DCI referral |
| `/odpp/referral/submit` | Submit ODPP referral |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| nationalId | A (Restricted) |
| fullName | A (Restricted) |
| criminalRecord | A (Restricted) |

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
| NCIC_DCI_ODPP_LATENCY_MS | 0 | ms delay added to every response |
| NCIC_DCI_ODPP_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
