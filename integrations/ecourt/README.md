# integrations/ecourt

eCourt petition filing and document handoff adapter for M40

## Wire protocol

Judiciary eCourt REST API; OAuth2; POST /petitions for filing, POST /documents for attachment upload.

### Endpoints

| Path | Description |
|---|---|
| `/ecourt/petition/file` | File election petition |
| `/ecourt/document/upload` | Upload supporting document |
| `/ecourt/case/status` | Check case status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| petitionerName | A (Restricted) |
| caseNo | B (Internal) |
| documentContent | B (Internal) |

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
| ECOURT_LATENCY_MS | 0 | ms delay added to every response |
| ECOURT_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
