# integrations/card-manufacturer

REST and EDI card personalization and issuance with manufacturer partner for election-related credential cards.

**MoU Status: Card Manufacturer NDA in place**

## Module usage

- Election observer credential card personalization
- Election official ID card printing and issuance
- Accreditation card manufacturing and delivery
- Batch card personalization and fulfillment

## Wire protocol and authentication

Protocol: REST (HTTPS) and EDI (X.400 or proprietary)
Authentication: mTLS (client certificate) + EDI partner ID
Endpoints: Card manufacturer production system

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/batch/submit` | POST | Submit personalization batch |
| `/batch/{batchId}` | GET | Get batch status and progress |
| `/cards/{cardId}` | GET | Get individual card production status |
| `/production/schedule` | GET | View production timeline |
| `/edi/shipment` | POST | EDI shipment data submission |
| `/quality/report` | GET | Quality assurance report |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| cardID | A (Restricted) | Unique card identifier |
| personName | A (Restricted) | Cardholder name (maximum sensitivity) |
| personID | A (Restricted) | National ID on card |
| credentials | A (Restricted) | Card credential data |
| batchCode | B (Internal) | Production batch identifier |
| productionDate | C (Public) | Manufacturing timestamp |

## Request and response shape

**Card Batch Submission Request:**
- batchName: string (required, descriptive)
- cardType: enum (observer, official, accreditation)
- cardQuantity: integer (required)
- personalizationData: array of {name, personID, title, organization}
- deliveryLocation: string (required)
- deliveryDate: ISO 8601 date (required)
- securityFeatures: array of strings (hologram, chip, signature panel)

**Card Batch Response:**
- batchID: string (UUID)
- status: enum (submitted, queued, production, quality_check, packaging, ready_for_pickup)
- estimatedCompletion: ISO 8601 datetime
- cardCount: integer
- trackingCode: string (for courier)

**Individual Card Status Response:**
- cardID: string
- batchID: string
- personName: string (first chars only)
- status: enum (personalized, quality_passed, packaged, shipped)
- serialNumber: string
- issueDate: ISO 8601 date

## Sample usage

```bash
# Submit card personalization batch (requires mTLS + EDI partner ID)
curl -X POST https://api.cardmfr.com/batch/submit \
  --cert $CARD_MFR_CERT --key $CARD_MFR_KEY \
  -H "X-EDI-Partner: $CARD_MFR_EDI_PARTNER_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "batchName": "Election Observers - August 2025",
    "cardType": "observer",
    "cardQuantity": 500,
    "personalizationData": [
      {
        "name": "Jane Mwangi",
        "personID": "12345678",
        "title": "International Observer",
        "organization": "African Union"
      }
    ],
    "deliveryLocation": "IEBC Headquarters, Nairobi",
    "deliveryDate": "2025-07-20",
    "securityFeatures": ["hologram", "signature_panel"]
  }'

# Track batch production
curl -X GET "https://api.cardmfr.com/batch/BATCH-2025-001" \
  --cert $CARD_MFR_CERT --key $CARD_MFR_KEY

# EDI shipment data
curl -X POST https://api.cardmfr.com/edi/shipment \
  --cert $CARD_MFR_CERT --key $CARD_MFR_KEY \
  -H "X-EDI-Partner: $CARD_MFR_EDI_PARTNER_ID" \
  -d @shipment.edi
```

## Health and readiness

```bash
curl -s https://api.cardmfr.com/health
# Response: {"status": "ready", "production_lines": 3, "queue": 12}
```

## Rate limits

100 batch submissions per minute per partner

| Metric | Limit | Window |
|---|---|---|
| Batch submissions | 100/min | rolling 1-minute |
| Status queries | 200/min | rolling 1-minute |
| EDI submissions | 50/min | rolling 1-minute |
| Bulk card lookups | 10/min | rolling 1-minute |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 1 error (critical production system) |
| open duration | 120 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `CARD_MFR_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue batch submission to secure offline file. EDI transmission via SFTP to partner secure mailbox. Manual coordination via phone hotline. Queue TTL: 24 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| CARD_MFR_BASE_URL | yes | - | Card manufacturer API base URL |
| CARD_MFR_CERT | yes | - | mTLS client cert (PEM) |
| CARD_MFR_KEY | yes | - | mTLS private key (PEM) |
| CARD_MFR_EDI_PARTNER_ID | yes | - | EDI partner identifier |
| CARD_MFR_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| CARD_MFR_TIMEOUT_SECS | no | 30 | Request timeout |

## Related modules

- Election credential and accreditation module - primary consumer
- Cardholder data management module - personalization data provider
- Logistics and delivery module - card fulfillment tracker

## Runbook

Refer to `docs/runbooks/integrations/card-manufacturer.md` for mTLS certificate and EDI partner ID rotation, and batch production contingency procedures.
