# integrations/mck

REST media and journalist accreditation for election coverage.

**MoU Status: Media Council of Kenya MoU pending**

## Module usage

- Media accreditation and journalist credential lookup during elections
- Press registration and accreditation verification

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: API-key
Endpoints: Media Council of Kenya (MCK) accreditation API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/accreditation/apply` | POST | Submit media outlet accreditation |
| `/accreditation/{id}` | GET | Get accreditation status |
| `/journalists/register` | POST | Register individual journalist |
| `/credentials/verify` | GET | Verify journalist credentials |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| journalistName | C (Public) | Public press roster |
| mediaOutlet | C (Public) | Public media outlet name |
| credentials | C (Public) | Press credentials (public during elections) |
| contactEmail | C (Public) | Media contact (public register) |
| phone | C (Public) | Press office phone (public) |

## Request and response shape

**Accreditation Request:**
- mediaOutlet: string (required, outlet legal name)
- outletType: enum (broadcast, print, digital, required)
- journalistName: string (required)
- journalistID: string (national ID, required)
- email: string (required)
- phone: string

**Accreditation Response:**
- accreditationID: string (UUID)
- status: enum (approved, pending, rejected)
- credentialNumber: string
- expiryDate: ISO 8601 date
- issuedDate: ISO 8601 timestamp

## Sample usage

```bash
# Register journalist for accreditation (requires API-key)
curl -X POST https://api.mck.or.ke/journalists/register \
  -H "X-API-Key: $MCK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "mediaOutlet": "Daily Times",
    "journalistName": "John Mwangi",
    "journalistID": "23456789",
    "email": "john@dailytimes.co.ke",
    "phone": "+254701234567"
  }'

# Verify credentials
curl -X GET "https://api.mck.or.ke/credentials/verify?credentialNumber=ACCR-2025-001" \
  -H "X-API-Key: $MCK_API_KEY"
```

## Health and readiness

```bash
curl -s https://api.mck.or.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

30 requests per minute per API key

| Metric | Limit | Window |
|---|---|---|
| New registrations | 10/day | calendar day |
| Lookups | 30/min | rolling 1-minute |
| Batch verifications | 5/hour | rolling hour |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 3 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `MCK_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue accreditation to fallback workflow. Journalist credentials sent via secure email to MCK with eel reference. Manual review and approval by MCK staff. Queue TTL: 24 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| MCK_BASE_URL | yes | - | MCK API base URL |
| MCK_API_KEY | yes | - | API key for authentication |
| MCK_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| MCK_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Election press operations module - primary consumer
- Credential verification module - validates journalist status

## Runbook

Refer to `docs/runbooks/integrations/mck.md` for API key rotation and accreditation workflows.
