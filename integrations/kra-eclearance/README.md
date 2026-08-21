# integrations/kra-eclearance

REST customs and tax clearance from Kenya Revenue Authority (KRA) with PIN authentication.

**MoU Status: KRA eClearance MoU signed**

## Module usage

- Tax compliance verification for candidates and officials
- Customs clearance for election material imports
- Revenue authority status checks

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS (client certificate) + PIN-based access
Endpoints: KRA eClearance API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/clearance/tax` | POST | Check tax compliance status |
| `/clearance/customs` | POST | Verify customs clearance |
| `/status/{pin}` | GET | Get clearance status by PIN |
| `/obligations/check` | GET | Check outstanding tax obligations |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| PIN | A (Restricted) | Personal identification number (maximum sensitivity) |
| taxStatus | A (Restricted) | Tax compliance details |
| obligations | A (Restricted) | Financial obligations data |
| customsRecords | A (Restricted) | Import/export history |
| complianceStatus | B (Internal) | Clearance determination |

## Request and response shape

**Clearance Request:**
- PIN: string (required, 11 digits)
- type: enum (individual, company, required)
- checkType: enum (tax_compliance, customs, full_audit)
- dateFrom: ISO 8601 date (optional)

**Clearance Response:**
- clearanceID: string (UUID)
- PIN: string (masked: ****12345)
- status: enum (compliant, non_compliant, under_review, suspended)
- taxOutstanding: decimal (KES amount)
- lastPayment: ISO 8601 date
- customsClearance: boolean
- checkDate: ISO 8601 timestamp
- validUntil: ISO 8601 date (48 hours typical)

## Sample usage

```bash
# Check tax clearance (requires mTLS + PIN)
curl -X POST https://api.kra.go.ke/clearance/tax \
  --cert $KRA_ECLEARANCE_CERT --key $KRA_ECLEARANCE_KEY \
  -H "Content-Type: application/json" \
  -H "X-PIN: $KRA_PIN_AUTH" \
  -d '{
    "PIN": "12345678901",
    "type": "individual",
    "checkType": "tax_compliance"
  }'

# Get clearance status
curl -X GET "https://api.kra.go.ke/status/12345678901" \
  --cert $KRA_ECLEARANCE_CERT --key $KRA_ECLEARANCE_KEY
```

## Health and readiness

```bash
curl -s https://api.kra.go.ke/health
# Response: {"status": "ready", "version": "2.0"}
```

## Rate limits

10 requests per minute per mTLS certificate

| Metric | Limit | Window |
|---|---|---|
| Clearance checks | 10/min | rolling 1-minute |
| Bulk queries | 2/hour | rolling hour |

Cache TTL: 48 hours for clearance status

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `KRA_ECLEARANCE_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue clearance check to offline workflow. Request sent to KRA eClearance team via secure channel with eel reference and PIN. Manual verification via KRA portal. Queue TTL: 72 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| KRA_ECLEARANCE_BASE_URL | yes | - | KRA API base URL |
| KRA_ECLEARANCE_CERT | yes | - | mTLS client cert (PEM) |
| KRA_ECLEARANCE_KEY | yes | - | mTLS private key (PEM) |
| KRA_PIN_AUTH | yes | - | PIN for authentication |
| KRA_ECLEARANCE_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| KRA_ECLEARANCE_CACHE_TTL_SECS | no | 172800 | Cache TTL (48 hours) |
| KRA_ECLEARANCE_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Candidate tax compliance module - primary consumer
- Official financial audit module - reference
- Election eligibility module - vetting

## Runbook

Refer to `docs/runbooks/integrations/kra-eclearance.md` for mTLS certificate rotation and PIN management.
