# integrations/nrb

REST national identity register verification from National Registration Bureau (NRB).

**MoU Status: NRB National ID Verification MoU signed**

## Module usage

- Voter identity verification against national ID register
- Candidate identity confirmation and eligibility vetting
- Election official background and identity validation

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS (client certificate)
Endpoints: NRB National ID Verification API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/identity/verify` | POST | Verify national ID against register |
| `/identity/{idNumber}` | GET | Get identity record by ID number |
| `/status/active` | GET | Check if ID is active/cancelled |
| `/records/search` | POST | Search by name and date of birth |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| idNumber | A (Restricted) | National ID number (maximum sensitivity) |
| fullName | A (Restricted) | Full legal name (identifying) |
| dateOfBirth | A (Restricted) | Birth date (identifying) |
| citizenship | A (Restricted) | Citizenship status (sensitive) |
| idStatus | B (Internal) | Active/cancelled/lost status |
| verificationDate | C (Public) | When verified |

## Request and response shape

**Identity Verification Request:**
- idNumber: string (required, 8 digits)
- fullName: string (required)
- dateOfBirth: ISO 8601 date (optional, for disambiguation)

**Identity Verification Response:**
- idNumber: string (masked: ****5678)
- status: enum (valid, cancelled, expired, not_found, duplicate)
- fullName: string
- dateOfBirth: ISO 8601 date
- citizenship: enum (kenyan, dual, foreign)
- issuedDate: ISO 8601 date
- expiryDate: ISO 8601 date
- verificationTimestamp: ISO 8601 timestamp
- cachedSince: ISO 8601 date (cache metadata)

## Sample usage

```bash
# Verify national ID (requires mTLS)
curl -X POST https://api.nrb.go.ke/identity/verify \
  --cert $NRB_CERT --key $NRB_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "idNumber": "12345678",
    "fullName": "Jane Mwangi",
    "dateOfBirth": "1985-03-20"
  }'

# Check ID status
curl -X GET "https://api.nrb.go.ke/status/active?idNumber=12345678" \
  --cert $NRB_CERT --key $NRB_KEY
```

## Health and readiness

```bash
curl -s https://api.nrb.go.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

30 requests per minute per mTLS certificate

| Metric | Limit | Window |
|---|---|---|
| Identity verifications | 30/min | rolling 1-minute |
| Status checks | 30/min | rolling 1-minute |
| Bulk searches | 2/hour | rolling hour |

Cache TTL: 4 hours for identity records

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `NRB_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: use cached identity records (TTL: 4 hours). For fresh verification, escalate to manual NRB confirmation via phone hotline. Queue TTL: 48 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| NRB_BASE_URL | yes | - | NRB API base URL |
| NRB_CERT | yes | - | mTLS client cert (PEM) |
| NRB_KEY | yes | - | mTLS private key (PEM) |
| NRB_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| NRB_CACHE_TTL_SECS | no | 14400 | Cache TTL (4 hours) |
| NRB_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Voter registration and eligibility module - primary consumer
- Candidate vetting and eligibility module - identity verification
- Election official verification module - personnel screening

## Runbook

Refer to `docs/runbooks/integrations/nrb.md` for mTLS certificate management and cache invalidation procedures.
