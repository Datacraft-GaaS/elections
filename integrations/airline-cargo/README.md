# integrations/airline-cargo

REST and EDI logistics manifest and cargo tracking via Kenya Airways (KQ).

**MoU Status: Kenya Airways (KQ) MoU signed**

## Module usage

- Ballot and election material transport logistics
- Cargo manifest and tracking via airline partner
- Electronic data interchange (EDI) for shipment data

## Wire protocol and authentication

Protocol: REST (HTTPS) and EDI (X.400 or proprietary)
Authentication: OAuth2 + X.509 certificate
Endpoints: Kenya Airways cargo REST API + EDI gateway

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/cargo/manifest` | POST | Create shipping manifest |
| `/cargo/{id}` | GET | Get cargo status and tracking |
| `/cargo/{id}/updates` | GET | Stream cargo updates |
| `/EDI/shipment` | POST | EDI shipment data (proprietary format) |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| shipmentID | C (Public) | Tracking number (public) |
| contents | B (Internal) | Cargo description |
| destinations | B (Internal) | Delivery locations (election sites) |
| sender | B (Internal) | Shipping origin |
| recipient | B (Internal) | Recipient info |

## Request and response shape

**Manifest Request:**
- shipmentType: enum (ballots, election_materials, equipment)
- contents: string (itemized list)
- weight: decimal (kg)
- dimensions: object (length, width, height)
- origin: string (airport/warehouse code)
- destination: string (airport/warehouse code)
- requiredDelivery: ISO 8601 date
- specialHandling: string (temperature control, fragile, etc.)

**Manifest Response:**
- manifestID: string (UUID)
- awbNumber: string (Air Waybill number)
- status: enum (booked, in_transit, delivered)
- estimatedDelivery: ISO 8601 datetime
- trackingURL: string
- flightNo: string (when assigned)

## Sample usage

```bash
# Create cargo manifest (requires OAuth2 token)
curl -X POST https://api.kq.co.ke/cargo/manifest \
  --cert $AIRLINE_CARGO_CERT --key $AIRLINE_CARGO_CERT_KEY \
  -H "Authorization: Bearer $AIRLINE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "shipmentType": "ballots",
    "contents": "Election ballots, county code 025",
    "weight": 500,
    "origin": "NBO",
    "destination": "MSA",
    "requiredDelivery": "2025-08-10"
  }'

# Track cargo
curl -X GET "https://api.kq.co.ke/cargo/KQ-2025-00123" \
  --cert $AIRLINE_CARGO_CERT --key $AIRLINE_CARGO_CERT_KEY
```

## Health and readiness

```bash
curl -s https://api.kq.co.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

30 requests per minute per OAuth2 client

| Metric | Limit | Window |
|---|---|---|
| Manifest creation | 30/min | rolling 1-minute |
| Tracking queries | 30/min | rolling 1-minute |
| EDI submissions | 10/hour | rolling hour |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `AIRLINE_CARGO_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: fall back to phone/email submission of manifests. Paper manifests sent via secure courier to KQ cargo operations. Manual processing by airline staff. Queue TTL: 12 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| AIRLINE_CARGO_BASE_URL | yes | - | Kenya Airways API base URL |
| AIRLINE_CARGO_CLIENT_ID | yes | - | OAuth2 client ID |
| AIRLINE_CARGO_CLIENT_SECRET | yes | - | OAuth2 client secret |
| AIRLINE_CARGO_TOKEN_URL | yes | - | OAuth2 token endpoint |
| AIRLINE_EDI_HOST | yes | - | EDI gateway hostname |
| AIRLINE_EDI_PORT | yes | - | EDI gateway port |
| AIRLINE_CARGO_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| AIRLINE_CARGO_TIMEOUT_SECS | no | 15 | Request timeout |

## Related modules

- Logistics planning module - primary consumer
- Materials dispatch module - shipping coordinator
- Election operations module - recipient tracking

## Runbook

Refer to `docs/runbooks/integrations/airline-cargo.md` for OAuth2 credential rotation and EDI gateway troubleshooting.
