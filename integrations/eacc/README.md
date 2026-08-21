# integrations/eacc

REST East African Community (EAC) clearance verification for regional mobility and freedom of movement compliance.

**MoU Status: EAC Clearance System MoU signed**

## Module usage

- Regional clearance for election staff mobility across EAC member states
- Freedom of movement verification for international observer teams
- Cross-border vetting for multinational election cooperation

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS (client certificate)
Endpoints: EAC Clearance and Movement Portal API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/clearance/eac` | POST | Verify EAC regional clearance |
| `/movement/{personId}` | GET | Check freedom of movement status |
| `/member-states/status` | GET | Get member state clearance requirements |
| `/travel-history` | GET | Query cross-border movement history |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| personID | B (Internal) | Individual identifier |
| movementClearance | B (Internal) | Regional movement status |
| travelHistory | B (Internal) | Cross-border movement records |
| memberStates | B (Internal) | Authorized countries |
| clearanceDate | C (Public) | Verification timestamp |

## Request and response shape

**Clearance Verification Request:**
- personID: string (required, national ID or passport)
- name: string (required)
- purpose: enum (election_observer, election_staff, voter_verification)
- travelPeriod: object {startDate, endDate, ISO 8601 dates}

**Clearance Verification Response:**
- clearanceID: string (UUID)
- status: enum (cleared, flagged, denied, pending)
- authorizedMemberStates: array of strings (country codes)
- clearanceExpiryDate: ISO 8601 date
- verificationDate: ISO 8601 timestamp
- notes: string (reason if flagged/denied)

## Sample usage

```bash
# Verify EAC clearance (requires mTLS)
curl -X POST https://api.eac.org/clearance/eac \
  --cert $EACC_CERT --key $EACC_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "personID": "12345678",
    "name": "Jane Mwangi",
    "purpose": "election_observer",
    "travelPeriod": {
      "startDate": "2025-08-01",
      "endDate": "2025-08-31"
    }
  }'

# Check freedom of movement status
curl -X GET "https://api.eac.org/movement/12345678" \
  --cert $EACC_CERT --key $EACC_KEY
```

## Health and readiness

```bash
curl -s https://api.eac.org/health
# Response: {"status": "ready", "memberStates": 8}
```

## Rate limits

10 requests per minute per mTLS certificate

| Metric | Limit | Window |
|---|---|---|
| Clearance checks | 10/min | rolling 1-minute |
| History queries | 5/min | rolling 1-minute |
| Bulk verifications | 2/hour | rolling hour |

Cache TTL: 48 hours for clearance status

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 1 error (conservative, regional system) |
| open duration | 120 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `EACC_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue clearance request to regional desk. Request sent to EAC Secretariat with eel reference. Manual regional coordination via secure channel. Queue TTL: 5 days.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| EACC_BASE_URL | yes | - | EAC API base URL |
| EACC_CERT | yes | - | mTLS client cert (PEM) |
| EACC_KEY | yes | - | mTLS private key (PEM) |
| EACC_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| EACC_CACHE_TTL_SECS | no | 172800 | Cache TTL (48 hours) |
| EACC_TIMEOUT_SECS | no | 15 | Request timeout |

## Related modules

- International observer coordination module - primary consumer
- Cross-border staff deployment module - movement authorization
- Regional election cooperation module - member state liaison

## Runbook

Refer to `docs/runbooks/integrations/eacc.md` for mTLS certificate rotation and member state status updates.
