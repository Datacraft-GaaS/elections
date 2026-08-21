# integrations/kps-kdf-unhcr

REST multi-agency verification across Kenya Police Service (KPS), Kenya Defence Force (KDF), and UNHCR (refugee status).

**MoU Status: All agencies (KPS, KDF, UNHCR) MoU signed**

## Module usage

- Police clearance verification for election officials
- Military/security screening for sensitive positions
- Refugee and asylum seeker status verification

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS client certificate per agency (separate cert for KPS, KDF, UNHCR)
Endpoints: Three separate agency verification systems

### Endpoints

| Agency | Path | Method | Description |
|---|---|---|---|
| KPS | `/clearance/police` | POST | Police clearance check |
| KPS | `/wanted/{personId}` | GET | Check wanted persons register |
| KDF | `/security/military` | POST | Military clearance check |
| KDF | `/records/{personId}` | GET | Military service records |
| UNHCR | `/refugee/status` | POST | Verify refugee/asylum status |
| UNHCR | `/persons/{personId}` | GET | Get UNHCR person record |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| personID | B (Internal) | Individual identifier |
| clearanceStatus | B (Internal) | Police/military clearance |
| refugeeStatus | B (Internal) | Asylum/refugee determination |
| recordDetails | B (Internal) | Agency-specific data |
| checkDate | C (Public) | Verification timestamp |

## Request and response shape

**Police Clearance Request:**
- personID: string (national ID, required)
- name: string (required)
- requestType: enum (clearance, wanted_check, background)

**Police Clearance Response:**
- status: enum (cleared, flagged, wanted, no_record)
- recordType: enum (criminal, traffic, administrative)
- severity: enum (minor, moderate, serious, critical)
- checkDate: ISO 8601 timestamp

**Military Clearance Request:**
- personID: string (required)
- name: string (required)

**Military Clearance Response:**
- militaryStatus: enum (none, served, active_reserve, court_martialed)
- serviceType: enum (army, air_force, navy)
- serviceEndDate: ISO 8601 date
- clearanceLevel: enum (none, secret, top_secret)

**UNHCR Status Request:**
- personID: string (required)
- documentType: enum (refugee_card, asylum_document, national_id)

**UNHCR Status Response:**
- refugeeStatus: enum (refugee, asylum_seeker, rejected, unknown)
- statusDate: ISO 8601 date
- countryOfOrigin: string
- statusExpiryDate: ISO 8601 date (if applicable)

## Sample usage

```bash
# Police clearance (requires KPS mTLS cert)
curl -X POST https://api.kps.go.ke/clearance/police \
  --cert $KPS_CERT --key $KPS_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "personID": "12345678",
    "name": "John Kipchoge",
    "requestType": "clearance"
  }'

# Military clearance (requires KDF mTLS cert)
curl -X POST https://api.kdf.go.ke/security/military \
  --cert $KDF_CERT --key $KDF_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "personID": "12345678",
    "name": "John Kipchoge"
  }'

# UNHCR status (requires UNHCR mTLS cert)
curl -X POST https://api.unhcr.go.ke/refugee/status \
  --cert $UNHCR_CERT --key $UNHCR_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "personID": "UNHCR-2024-001234",
    "documentType": "refugee_card"
  }'
```

## Health and readiness

```bash
curl -s https://api.kps.go.ke/health
# Response: {"status": "ready", "agencies": ["kps", "kdf", "unhcr"]}
```

## Rate limits

10 requests per minute per agency (separate limits per KPS, KDF, UNHCR)

| Agency | Metric | Limit | Window |
|---|---|---|---|
| KPS | Clearance checks | 10/min | rolling 1-minute |
| KDF | Security checks | 10/min | rolling 1-minute |
| UNHCR | Status queries | 10/min | rolling 1-minute |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (per agency) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

Each agency has independent circuit breaker.

## Chaos testing

Set `KPS_KDF_UNHCR_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open per agency: queue verification request. For KPS/KDF, escalate to security team phone line. For UNHCR, contact via secure email to regional office. Queue TTL: 72 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| KPS_BASE_URL | yes | - | KPS API base URL |
| KPS_CERT | yes | - | KPS mTLS client cert (PEM) |
| KPS_KEY | yes | - | KPS mTLS private key (PEM) |
| KDF_BASE_URL | yes | - | KDF API base URL |
| KDF_CERT | yes | - | KDF mTLS client cert (PEM) |
| KDF_KEY | yes | - | KDF mTLS private key (PEM) |
| UNHCR_BASE_URL | yes | - | UNHCR API base URL |
| UNHCR_CERT | yes | - | UNHCR mTLS client cert (PEM) |
| UNHCR_KEY | yes | - | UNHCR mTLS private key (PEM) |
| KPS_KDF_UNHCR_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| KPS_KDF_UNHCR_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Election official vetting module - primary consumer
- Security position clearance module - multi-agency validator
- Conflict of interest screening module - police/military records

## Runbook

Refer to `docs/runbooks/integrations/kps-kdf-unhcr.md` for certificate management across three agencies and multi-agency circuit breaker troubleshooting.
