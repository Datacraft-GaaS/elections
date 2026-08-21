# integrations/ncic-dci-odpp

Security clearance check across three law enforcement agencies: NCIC (National Crime Intelligence Center), DCI (Directorate of Criminal Investigations), and ODPP (Office of the Director of Public Prosecutions).

**MoU Status: All agencies (NCIC, DCI, ODPP) MoU signed**

## Module usage

- Security vetting for election officials and sensitive positions
- Criminal clearance verification across three independent agencies

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS client certificate per agency (separate cert for NCIC, DCI, ODPP)
Endpoints: Three separate agency endpoints

### Endpoints

| Agency | Path | Method | Description |
|---|---|---|---|
| NCIC | `/clearance/check` | POST | Query criminal record clearance |
| NCIC | `/records/{id}` | GET | Retrieve clearance status by ID |
| DCI | `/investigations/status` | POST | Check investigation status |
| DCI | `/wanted/{id}` | GET | Check wanted persons register |
| ODPP | `/prosecution/cleared` | GET | Verify prosecution clearance |
| ODPP | `/cases/{id}` | GET | Check case involvement |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| fullName | A (Restricted) | Subject name and identity |
| nationalID | A (Restricted) | ID number (sensitive) |
| crimeRecord | A (Restricted) | Criminal history (maximum sensitivity) |
| clearanceStatus | A (Restricted) | Security determination |
| checkDate | B (Internal) | Timestamp of check |
| requestingAgency | B (Internal) | Who requested clearance |

## Request and response shape

**Clearance Request:**
- nationalID: string (required, 8-10 digits)
- fullName: string (required)
- dateOfBirth: ISO 8601 date
- requestingAgency: string (required, enum: electoral_commission, security_board)

**Clearance Response:**
- clearanceID: string (UUID)
- status: enum (cleared, flagged, investigation_active, no_record)
- agenciesChecked: array of strings (ncic, dci, odpp)
- checkDate: ISO 8601 timestamp
- expiryDate: ISO 8601 date (typically 2 years)
- flagDetails: string (when status is flagged)

## Sample usage

```bash
# Check clearance (requires separate mTLS certs per agency)
curl -X POST https://api.ncic.go.ke/clearance/check \
  --cert $NCIC_CERT --key $NCIC_KEY \
  -H "Content-Type: application/json" \
  -d '{
    "nationalID": "12345678",
    "fullName": "Jane Mwangi",
    "dateOfBirth": "1985-06-15",
    "requestingAgency": "electoral_commission"
  }'

# Check wanted persons across DCI
curl -X GET "https://api.dci.go.ke/wanted/12345678" \
  --cert $DCI_CERT --key $DCI_KEY
```

## Health and readiness

```bash
curl -s https://api.ncic.go.ke/health
# Response: {"status": "ready", "agencies": ["ncic", "dci", "odpp"]}
```

## Rate limits

10 requests per minute per agency (separate limits per NCIC, DCI, ODPP)

| Agency | Metric | Limit | Window |
|---|---|---|---|
| NCIC/DCI/ODPP | Clearance checks | 10/min | rolling 1-minute |
| NCIC/DCI/ODPP | Bulk queries | 2/hour | rolling hour |

Cache TTL: 48 hours for clearance status

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 1 error (per agency) triggers open |
| open duration | 120 seconds |
| retry policy | exponential backoff (base 500ms, max 10s) |

Note: Conservative 1-fail threshold due to security sensitivity. Each agency has independent circuit breaker.

## Chaos testing

Set `NCIC_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue clearance check to offline processing. Request sent to security operations team with eel reference. Manual verification across agencies via phone/secure mail. Queue TTL: 72 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| NCIC_BASE_URL | yes | - | NCIC API base URL |
| NCIC_CERT | yes | - | NCIC mTLS client cert (PEM) |
| NCIC_KEY | yes | - | NCIC mTLS private key (PEM) |
| DCI_BASE_URL | yes | - | DCI API base URL |
| DCI_CERT | yes | - | DCI mTLS client cert (PEM) |
| DCI_KEY | yes | - | DCI mTLS private key (PEM) |
| ODPP_BASE_URL | yes | - | ODPP API base URL |
| ODPP_CERT | yes | - | ODPP mTLS client cert (PEM) |
| ODPP_KEY | yes | - | ODPP mTLS private key (PEM) |
| NCIC_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| NCIC_CACHE_TTL_SECS | no | 172800 | Cache TTL (48 hours) |

## Related modules

- Election official vetting module - primary consumer
- Security position clearance module - validators

## Runbook

Refer to `docs/runbooks/integrations/ncic-dci-odpp.md` for credential management and circuit breaker recovery procedures. Three separate certificates must be maintained.
