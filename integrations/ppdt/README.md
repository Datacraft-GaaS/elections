# integrations/ppdt

PPDT political parties registration and compliance adapter

## Wire protocol

PPDT REST API; API key; endpoints for party lookup, coalition registration, compliance check.

### Endpoints

| Path | Description |
|---|---|
| `/ppdt/party/lookup` | Lookup party details |
| `/ppdt/coalition/register` | Register coalition |
| `/ppdt/compliance/check` | Check compliance status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| partyName | C (Public) |
| registrationNo | B (Internal) |
| officerName | A (Restricted) |

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
| PPDT_LATENCY_MS | 0 | ms delay added to every response |
| PPDT_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
