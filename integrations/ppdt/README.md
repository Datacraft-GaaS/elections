# integrations/ppdt

REST party registration and details lookup for political parties.

**MoU Status: Party Petitioning Database MoU signed**

## Module usage

- Party registration and details lookup across integrated party systems

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: OAuth2
Endpoints: Party Petitioning Database (PPDT) REST API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/parties/register` | POST | Register political party |
| `/parties/{id}` | GET | Get party details and registration status |
| `/parties/search` | GET | Search parties by name or ID |
| `/parties/{id}/members` | GET | List registered party members |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| partyName | C (Public) | Party legal name (public register) |
| leaderName | B (Internal) | Party leader identity |
| memberNames | B (Internal) | Party member roster |
| registrationDate | C (Public) | Public registration timestamp |
| contactEmail | A (Restricted) | Party contact details |

## Request and response shape

**Register Party Request:**
- partyName: string (required, 3-100 chars)
- leaderName: string (required)
- leaderID: string (national ID, required)
- partyAcronym: string (required, 2-5 chars)
- registrationFee: decimal
- contactEmail: string
- contactPhone: string

**Register Party Response:**
- partyID: string (UUID)
- status: enum (pending_review, registered, rejected)
- registrationDate: ISO 8601 timestamp
- approvalDate: ISO 8601 timestamp (when applicable)

## Sample usage

```bash
# Register a political party (requires OAuth2 token)
curl -X POST https://api.ppdt.go.ke/parties/register \
  -H "Authorization: Bearer $PPDT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "partyName": "Democratic Alliance Party",
    "leaderName": "Jane Smith",
    "leaderID": "12345678",
    "partyAcronym": "DAP",
    "registrationFee": 50000,
    "contactEmail": "info@dap.co.ke",
    "contactPhone": "+254712345678"
  }'

# Get party details
curl -X GET https://api.ppdt.go.ke/parties/PAR-2025-001 \
  -H "Authorization: Bearer $PPDT_TOKEN"
```

## Health and readiness

```bash
curl -s https://api.ppdt.go.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

20 requests per minute per OAuth2 client

| Metric | Limit | Window |
|---|---|---|
| Party registrations | 5/day | calendar day |
| Lookups | 20/min | rolling 1-minute |
| Member list queries | 10/day | calendar day |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `PPDT_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue registration to fallback email workflow. Party details sent to PPDT registry with eel reference. Manual intake by registration staff. Queue TTL: 48 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| PPDT_BASE_URL | yes | - | PPDT API base URL |
| PPDT_CLIENT_ID | yes | - | OAuth2 client ID |
| PPDT_CLIENT_SECRET | yes | - | OAuth2 client secret |
| PPDT_TOKEN_URL | yes | - | OAuth2 token endpoint |
| PPDT_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| PPDT_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Party affiliation module - primary consumer
- Election candidacy module - validates party membership

## Runbook

Refer to `docs/runbooks/integrations/ppdt.md` for credential rotation and OAuth2 token management.
