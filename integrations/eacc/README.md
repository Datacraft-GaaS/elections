# integrations/eacc

EACC Ethics and Anti-Corruption Commission clearance adapter for M12 nominations

## Wire protocol

EACC REST API; mTLS; POST /clearance/check returns integrity certificate status.

### Endpoints

| Path | Description |
|---|---|
| `/eacc/clearance/check` | Check EACC integrity clearance |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| nationalId | A (Restricted) |
| fullName | A (Restricted) |
| clearanceStatus | B (Internal) |

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
| EACC_LATENCY_MS | 0 | ms delay added to every response |
| EACC_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
