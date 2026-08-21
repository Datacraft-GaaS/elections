# integrations/nllb

NLLB neural machine translation adapter for M17 and M28 multilingual

## Wire protocol

NLLB REST service; POST /translate with source_lang, target_lang, text; supports Swahili, Kikuyu, Dholuo.

### Endpoints

| Path | Description |
|---|---|
| `/nllb/translate` | Translate text between languages |
| `/nllb/detect` | Detect language |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| inputText | B (Internal) |
| translatedText | B (Internal) |

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
| NLLB_LATENCY_MS | 0 | ms delay added to every response |
| NLLB_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
