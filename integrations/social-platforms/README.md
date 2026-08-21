# integrations/social-platforms

Social media platform content flag and CIB detection adapter for M26

## Wire protocol

Meta/X/TikTok APIs via unified REST proxy; Bearer token; POST /report/flag, GET /content/status.

### Endpoints

| Path | Description |
|---|---|
| `/social/content/flag` | Flag social media content |
| `/social/content/status` | Check content moderation status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| contentUrl | B (Internal) |
| platformUserId | A (Restricted) |

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
| SOCIAL_PLATFORMS_LATENCY_MS | 0 | ms delay added to every response |
| SOCIAL_PLATFORMS_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
