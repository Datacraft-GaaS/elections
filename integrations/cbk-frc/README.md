# integrations/cbk-frc

Financial verification and compliance checking across Central Bank of Kenya (CBK) and Financial Regulatory Compliance (FRC).

**MoU Status: CBK MoU signed; FRC MoU pending**

## Module usage

- Financial compliance verification for candidates and officials
- Anti-money laundering (AML) and sanctions list checks
- Bank account and credit history verification

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS (client certificate per provider) + HMAC-SHA256 request signing
Endpoints: Two separate financial institutions (CBK and FRC)

### Endpoints

| Provider | Path | Method | Description |
|---|---|---|---|
| CBK | `/accounts/verify` | POST | Verify bank account holder |
| CBK | `/sanctions/check` | GET | Check sanctions list |
| FRC | `/compliance/status` | GET | Get compliance rating |
| FRC | `/aml/check` | POST | AML (anti-money laundering) check |
| FRC | `/credit/history` | GET | Check credit history |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| accountNumber | A (Restricted) | Bank account details |
| accountHolder | A (Restricted) | Individual/entity name |
| balanceInfo | A (Restricted) | Financial balance data (high sensitivity) |
| creditScore | B (Internal) | Credit rating |
| complianceStatus | B (Internal) | AML/compliance determination |
| checkDate | C (Public) | Timestamp of inquiry |

## Request and response shape

**Verification Request:**
- nationalID: string (required)
- fullName: string (required)
- accountNumber: string (optional, for specific account check)
- requestType: enum (full_compliance, sanctions_only, credit_check, required)

**Verification Response:**
- verificationID: string (UUID)
- complianceStatus: enum (compliant, flagged, blocked, unknown)
- amlStatus: enum (passed, review_required, failed)
- sanctionedEntity: boolean
- creditRating: string (when available)
- checkDate: ISO 8601 timestamp
- expiryDate: ISO 8601 date (90 days typical)

## Sample usage

```bash
# Check financial compliance (requires mTLS + HMAC signature)
curl -X POST https://api.cbk.go.ke/accounts/verify \
  --cert $CBK_CERT --key $CBK_KEY \
  -H "Content-Type: application/json" \
  -H "X-HMAC-SHA256: $(echo -n 'request_body' | openssl dgst -sha256 -hmac $CBK_HMAC_SECRET | tr -d ' ' | sed 's/^.*= //')" \
  -d '{
    "nationalID": "12345678",
    "fullName": "James Kipchoge",
    "requestType": "full_compliance"
  }'

# Check sanctions list
curl -X GET "https://api.cbk.go.ke/sanctions/check?nationalID=12345678" \
  --cert $CBK_CERT --key $CBK_KEY
```

## Health and readiness

```bash
curl -s https://api.cbk.go.ke/health
# Response: {"status": "ready", "providers": ["cbk", "frc"]}
```

## Rate limits

20 requests per minute per authentication pair

| Metric | Limit | Window |
|---|---|---|
| Compliance checks | 20/min | rolling 1-minute |
| Sanctions queries | 30/min | rolling 1-minute |
| Bulk verifications | 5/hour | rolling hour |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `CBK_FRC_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue verification to offline workflow. Financial status request sent to compliance team with eel reference. Manual verification with CBK/FRC via secure phone line. Queue TTL: 24 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| CBK_BASE_URL | yes | - | CBK API base URL |
| CBK_CERT | yes | - | CBK mTLS client cert (PEM) |
| CBK_KEY | yes | - | CBK mTLS private key (PEM) |
| CBK_HMAC_SECRET | yes | - | HMAC-SHA256 shared secret for CBK |
| FRC_BASE_URL | yes | - | FRC API base URL |
| FRC_CERT | yes | - | FRC mTLS client cert (PEM) |
| FRC_KEY | yes | - | FRC mTLS private key (PEM) |
| CBK_FRC_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| CBK_FRC_TIMEOUT_SECS | no | 15 | Request timeout |

## Related modules

- Candidate financial disclosure module - primary consumer
- Official wealth verification module - compliance check
- Election compliance auditing module - reference

## Runbook

Refer to `docs/runbooks/integrations/cbk-frc.md` for mTLS certificate and HMAC secret rotation procedures.
