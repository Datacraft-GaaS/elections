# integrations/crs

REST civil registration system (birth/death records) from National Registration Bureau (NRB).

**MoU Status: NRB Civil Registration System MoU signed**

## Module usage

- Voter eligibility verification via birth and death records
- Age verification for candidate eligibility
- Identity confirmation for election officials

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS (client certificate)
Endpoints: NRB Civil Registration System API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/registry/birth/verify` | POST | Verify birth record |
| `/registry/death/check` | POST | Check death record (exclusion) |
| `/registry/{regNumber}` | GET | Get registration details |
| `/records/search` | POST | Search by name and date of birth |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| fullName | A (Restricted) | Birth certificate name (maximum sensitivity) |
| dateOfBirth | A (Restricted) | Birth date (identifying) |
| birthCertNumber | A (Restricted) | Certificate number (sensitive ID) |
| parents | A (Restricted) | Parent names (genealogical PII) |
| registrationDate | B (Internal) | Registration timestamp |
| status | C (Public) | Active/deceased status |

## Request and response shape

**Birth Verification Request:**
- fullName: string (required, as per birth certificate)
- dateOfBirth: ISO 8601 date (required)
- birthCertNumber: string (optional, 15 digits)
- parentName: string (optional, for disambiguation)

**Birth Verification Response:**
- birthCertNumber: string
- status: enum (registered, not_registered, pending_verification)
- registrationDate: ISO 8601 date
- birthPlace: string (county/country)
- verificationTimestamp: ISO 8601 timestamp
- cachedSince: ISO 8601 date (cache metadata)

**Death Check Response:**
- isDeceased: boolean
- deathDate: ISO 8601 date (when applicable)
- deathCertNumber: string

## Sample usage

```bash
# Verify birth record (requires mTLS)
curl -X POST https://api.crs.go.ke/registry/birth/verify \
  --cert $CRS_CERT --key $CRS_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "fullName": "Jane Mwangi",
    "dateOfBirth": "1990-05-15",
    "parentName": "James Mwangi"
  }'

# Check death records
curl -X POST https://api.crs.go.ke/registry/death/check \
  --cert $CRS_CERT --key $CRS_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "fullName": "John Kipchoge",
    "dateOfBirth": "1960-01-01"
  }'
```

## Health and readiness

```bash
curl -s https://api.crs.go.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

60 requests per minute per mTLS certificate

| Metric | Limit | Window |
|---|---|---|
| Verification queries | 60/min | rolling 1-minute |
| Search queries | 40/min | rolling 1-minute |
| Bulk uploads | 5/hour | rolling hour |

Cache TTL: 24 hours for verified records

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `CRS_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: use cached birth/death records (TTL: 24 hours). For new queries, escalate to manual NRB verification via secure phone line. Queue TTL: 72 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| CRS_BASE_URL | yes | - | NRB API base URL |
| CRS_CERT | yes | - | mTLS client cert (PEM) |
| CRS_KEY | yes | - | mTLS private key (PEM) |
| CRS_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| CRS_CACHE_TTL_SECS | no | 86400 | Cache TTL (24 hours) |
| CRS_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Voter registration and eligibility module - primary consumer
- Candidate vetting module - age/identity verification
- Election official verification module - background check

## Runbook

Refer to `docs/runbooks/integrations/crs.md` for mTLS certificate rotation and cache invalidation procedures.
