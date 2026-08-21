# integrations/litellm

OpenAI-compatible LLM proxy hosted on-premises for chat and embeddings with optional Ollama fallback.

**MoU Status: Self-hosted (no external MoU required)**

## Module usage

- Natural language processing for election content and communications
- Semantic analysis of candidate statements and manifestos
- Text classification and information extraction
- Chatbot responses for voter FAQs

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: API-key
Endpoints: Self-hosted LiteLLM proxy at 62.169.25.77:4000

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/v1/chat/completions` | POST | OpenAI-compatible chat endpoint |
| `/v1/embeddings` | POST | Generate text embeddings |
| `/v1/models` | GET | List available models |
| `/health` | GET | Health check endpoint |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| prompt | C (Public) | User input (stripped of identifying info) |
| completion | C (Public) | LLM response (no stored identity) |
| embeddings | C (Public) | Vector representation (no PII) |
| modelID | C (Public) | Model identifier |
| timestamp | C (Public) | Request time |

## Request and response shape

**Chat Completion Request:**
- model: string (required, e.g., "gpt-3.5-turbo")
- messages: array of {role, content} (required)
- temperature: float (0.0-2.0, default 0.7)
- max_tokens: integer (optional)
- top_p: float (optional)

**Chat Completion Response:**
- id: string (UUID)
- object: string ("chat.completion")
- created: integer (unix timestamp)
- model: string
- choices: array [{message: {role, content}, finish_reason}]
- usage: {prompt_tokens, completion_tokens, total_tokens}

## Sample usage

```bash
# Chat completion (requires API-key)
curl -X POST http://62.169.25.77:4000/v1/chat/completions \
  -H "Authorization: Bearer $LITELLM_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Summarize election candidate profiles"}
    ],
    "temperature": 0.7,
    "max_tokens": 500
  }'

# Generate embeddings
curl -X POST http://62.169.25.77:4000/v1/embeddings \
  -H "Authorization: Bearer $LITELLM_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "text-embedding-ada-002",
    "input": "election manifesto text"
  }'
```

## Health and readiness

```bash
curl -s http://62.169.25.77:4000/health
# Response: {"status": "ready", "models": 5}
```

## Rate limits

No upstream rate limits (self-hosted). Application-level limits:

| Metric | Limit | Window |
|---|---|---|
| Chat requests | 100/min | rolling 1-minute |
| Embedding requests | 200/min | rolling 1-minute |
| Max tokens per request | 2000 | per request |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 3 consecutive errors (5xx, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `LITELLM_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: fall back to on-device Ollama instance running smaller model (Phi 2 or Mistral). Reduced capability but maintains service availability. Responses marked as fallback with model identifier.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| LITELLM_HOST | yes | 62.169.25.77 | LiteLLM proxy hostname |
| LITELLM_PORT | yes | 4000 | LiteLLM proxy port |
| LITELLM_API_KEY | yes | - | API key for authentication |
| LITELLM_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| LITELLM_TIMEOUT_SECS | no | 30 | Request timeout |
| OLLAMA_FALLBACK_HOST | no | localhost | Ollama fallback host |
| OLLAMA_FALLBACK_PORT | no | 11434 | Ollama fallback port |

## Related modules

- Election content analysis module - primary consumer
- Candidate manifesto processing module - text extraction
- Voter FAQ chatbot module - response generation

## Runbook

Refer to `docs/runbooks/integrations/litellm.md` for model configuration and Ollama fallback setup.
