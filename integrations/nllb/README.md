# integrations/nllb

Neural machine translation via Meta's NLLB-200 model for 200+ language pairs.

**MoU Status: Open source integration (no MoU required)**

## Module usage

- M54 (document-translate) - translate election materials, ballots, and notices
- M71 (accessibility-lang) - provide translated content for multilingual voters
- M89 (candidate-platform-translate) - translate candidate statements and manifestos

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: Bearer token
Endpoints: NLLB translation service (self-hosted or cloud deployment)

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/translate` | POST | Translate text between language pairs |
| `/detect` | POST | Auto-detect source language |
| `/languages` | GET | List supported language codes |
| `/health` | GET | Service health check |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| sourceText | B (Internal) | Content being translated (may contain PII) |
| targetLanguage | C (Public) | Language code (non-sensitive) |
| translatedText | B (Internal) | Translated output (preserves source sensitivity) |
| confidence | C (Public) | Translation confidence score |

## Request and response shape

**Translation Request:**
- sourceText: string (required, 1-10000 chars)
- sourceLanguage: string (required, ISO 639-3 code, or "auto" for detection)
- targetLanguage: string (required, ISO 639-3 code)
- model: string (default: "nllb-200-1.3B")

**Translation Response:**
- translatedText: string
- sourceLanguage: string (actual detected language if auto)
- targetLanguage: string
- confidence: number (0.0-1.0)
- tokensUsed: integer

## Sample usage

```bash
# Translate English to Swahili (requires Bearer token)
curl -X POST https://nllb.example.com/translate \
  -H "Authorization: Bearer $NLLB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "sourceText": "Voting stations open at 6 AM",
    "sourceLanguage": "eng_Latn",
    "targetLanguage": "swh_Latn"
  }'

# Auto-detect and translate
curl -X POST https://nllb.example.com/translate \
  -H "Authorization: Bearer $NLLB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "sourceText": "Les urnes ferment a 5 PM",
    "sourceLanguage": "auto",
    "targetLanguage": "swh_Latn"
  }'
```

## Health and readiness

```bash
curl -s https://nllb.example.com/health
# Response: {"status": "ready", "model": "nllb-200-1.3B", "capacity": "95%"}

curl -s https://nllb.example.com/languages | jq '.supportedLanguages | length'
# Response: 200
```

## Rate limits

100 requests per minute per Bearer token

| Metric | Limit | Window |
|---|---|---|
| Translation requests | 100/min | rolling 1-minute |
| Chars per request | 10000 | per request |
| Daily translation volume | 10M chars | calendar day |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors (5xx, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `NLLB_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 20s |
| error | return 503 Service Unavailable |
| garbled | return corrupted translation output |

## Fallback strategy

Circuit open: queue translation requests. Use cached translations for identical sourceText + language pair (TTL: 7 days). For new content, use fallback English or disable translation feature. Queue TTL: 24 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| NLLB_BASE_URL | yes | - | NLLB service base URL |
| NLLB_TOKEN | yes | - | Bearer token for authentication |
| NLLB_MODEL | no | nllb-200-1.3B | Model variant to use |
| NLLB_TIMEOUT_SECS | no | 20 | Request timeout |
| NLLB_CHAOS | no | false | Chaos mode (timeout/error/garbled) |

## Related modules

- M54 (document-translate) - primary consumer
- M71 (accessibility-lang) - language support provider
- M89 (candidate-platform-translate) - manifesto translation

## Runbook

Refer to `docs/runbooks/integrations/nllb.md` for model updates and language-pair compatibility troubleshooting.
