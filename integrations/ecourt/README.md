# integrations/ecourt

Judiciary eCourt petition filing and case document handoff for election court disputes.

**MoU Status: eCourt Judiciary MoU signed**

## Module usage

- M40 (election-court-it-interface) - petition filing and case tracking
- M36 (dereg-appeal) - deregistration appeal workflow

## Wire protocol and authentication

Protocol: REST (primary) and SOAP (legacy) over HTTPS
Authentication: mTLS client certificate plus session bearer token
Endpoints: Judiciary eCourt REST API and legacy SOAP service

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/ecourt/petition/file` | POST | File election petition |
| `/ecourt/document/upload` | POST | Upload supporting evidence document |
| `/ecourt/case/status` | GET | Check case status and docket number |
| `/ecourt/case/{caseNo}/update` | PATCH | Update case notes (internal use) |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| petitionerName | A (Restricted) | Citizen identity |
| respondentName | A (Restricted) | Citizen identity |
| caseNo | B (Internal) | Case identifier |
| documentContent | B (Internal) | Evidence content |
| filingDate | C (Public) | Case metadata |

## Request and response shape

**File Petition Request:**
- petitionerName: string (required)
- respondentName: string (required)
- grounds: enum (irregularity, fraud, malpractice)
- briefStatement: string (500-5000 chars)
- requestedRelief: string

**File Petition Response:**
- caseNo: string (eCourt docket number)
- filingDate: ISO 8601 timestamp
- estimatedHearing: ISO 8601 date
- status: enum (filed, assigned_judge, hearing_scheduled)

## Sample usage

```bash
# File petition (requires valid mTLS cert and session token)
curl -X POST https://api.ecourt.go.ke/petitions \
  --cert $ECOURT_CERT --key $ECOURT_KEY \
  -H "Authorization: Bearer $ECOURT_SESSION_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "petitionerName": "Jane Doe",
    "respondentName": "election_officer",
    "grounds": "irregularity",
    "briefStatement": "Polling agents denied observation rights",
    "requestedRelief": "Recount and nullification"
  }'

# Check case status
curl -X GET https://api.ecourt.go.ke/cases/ECR-2025-001 \
  --cert $ECOURT_CERT --key $ECOURT_KEY \
  -H "Authorization: Bearer $ECOURT_SESSION_TOKEN"
```

## Health and readiness

```bash
curl -s https://api.ecourt.go.ke/health
# Response: {"status": "ready", "version": "2.1"}
```

## Rate limits

30 requests per minute per client certificate

| Metric | Limit | Window |
|---|---|---|
| Petition filings | 10/day | calendar day |
| Document uploads | 100/day | calendar day |
| Case queries | 30/min | rolling 1-minute |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 10s) |

## Chaos testing

Set `ECOURT_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue petition to email-to-court workflow. PDF bundle and covering letter sent to eCourt registry email with eel case reference. Manual intake and docket assignment by court staff. Queue TTL: 24 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| ECOURT_BASE_URL | yes | - | eCourt API base URL |
| ECOURT_CERT | yes | - | mTLS client certificate (PEM) |
| ECOURT_KEY | yes | - | mTLS private key (PEM) |
| ECOURT_SESSION_TOKEN | yes | - | Bearer token for session auth |
| ECOURT_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| ECOURT_TIMEOUT_SECS | no | 10 | Request timeout |
| ECOURT_CACHE_TTL_SECS | no | 3600 | Case status cache TTL |

## Related modules

- M40 (election-court-it-interface) - primary consumer
- M36 (dereg-appeal) - deregistration workflow
- M23 (grievance-evidence) - may reference eCourt cases

## Runbook

Refer to `docs/runbooks/integrations/ecourt.md` for circuit breaker recovery procedures and credential rotation.
