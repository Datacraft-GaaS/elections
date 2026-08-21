# integrations/power-bi

Power BI push dataset adapter for election results dashboards

## Wire protocol

Power BI REST API; OAuth2 MSAL; POST /datasets/{id}/tables/{table}/rows to stream rows.

### Endpoints

| Path | Description |
|---|---|
| `/powerbi/rows/push` | Push rows to Power BI dataset |
| `/powerbi/dataset/refresh` | Trigger dataset refresh |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| rowData | B (Internal) |

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
| POWER_BI_LATENCY_MS | 0 | ms delay added to every response |
| POWER_BI_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
