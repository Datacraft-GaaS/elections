# integrations/litellm

LiteLLM tenant proxy adapter for M17 chatbot and M28 AI flows

## Wire protocol

LiteLLM proxy REST API (OpenAI-compatible); Bearer token; POST /v1/chat/completions, POST /v1/embeddings.

### Endpoints

| Path | Description |
|---|---|
| `/litellm/chat/completions` | LLM chat completion |
| `/litellm/embeddings` | Generate text embeddings |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| messages | B (Internal) |
| systemPrompt | B (Internal) |

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
| LITELLM_LATENCY_MS | 0 | ms delay added to every response |
| LITELLM_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
