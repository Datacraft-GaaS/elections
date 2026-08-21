# integrations/nhif-shif

NHIF/SHA health insurance contributions and eligibility adapter

## Wire protocol

NHIF REST API; OAuth2; endpoints: eligibility check, contribution submission, claim status.

### Endpoints

| Path | Description |
|---|---|
| `/nhif/eligibility` | Check NHIF eligibility |
| `/nhif/contribution` | Submit contribution |
| `/nhif/claim/status` | Check claim status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| nationalId | A (Restricted) |
| memberNo | A (Restricted) |
| medicalHistory | A (Restricted) |

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
| NHIF_SHIF_LATENCY_MS | 0 | ms delay added to every response |
| NHIF_SHIF_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
