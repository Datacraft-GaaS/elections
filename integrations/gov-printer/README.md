# integrations/gov-printer

Government printer Gazette submission adapter for M52 and M40

## Wire protocol

Dual transport: SFTP for bulk gazette PDFs; REST POST /gazette/submit for metadata + file reference.

### Endpoints

| Path | Description |
|---|---|
| `/gov-printer/gazette/submit` | Submit gazette document |
| `/gov-printer/gazette/status` | Check submission status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| documentRef | B (Internal) |
| gazetteNoticeNo | B (Internal) |

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
| GOV_PRINTER_LATENCY_MS | 0 | ms delay added to every response |
| GOV_PRINTER_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
