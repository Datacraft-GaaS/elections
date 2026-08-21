# integrations/kebs-warranty

Manufacturing and consumer goods warranty verification for election materials and equipment.

**MoU Status: Kenya Bureau of Standards warranty program agreement in place**

## Module usage

- M66 (equipment-compliance) - verify voter machines and ballot scanners are under valid warranty
- M77 (materials-warranty) - track ballot printing and election materials warranty
- M83 (supply-chain-verify) - validate approved supplier registration and warranty coverage

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: API-key + manufacturer registration number
Endpoints: KEBS Warranty and Standards Portal API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/warranty/lookup` | POST | Look up product warranty status by serial number |
| `/warranty/verify` | POST | Verify warranty validity for specific date range |
| `/equipment/registered` | GET | Check if equipment is registered with KEBS |
| `/manufacturer/status` | GET | Get manufacturer compliance and warranty status |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| serialNumber | C (Public) | Equipment serial (public identifier) |
| productModel | C (Public) | Product model name (non-sensitive) |
| warrantyStatus | C (Public) | Warranty validity (non-sensitive) |
| manufacturerReg | C (Public) | Manufacturer registration (public record) |
| purchaseDate | C (Public) | Acquisition date (non-sensitive) |

## Request and response shape

**Warranty Lookup Request:**
- serialNumber: string (required, 10-50 chars)
- productCategory: string (required, enum: ballot-scanner, ballot-printer, polling-machine)
- registrationNumber: string (required, manufacturer ID)

**Warranty Lookup Response:**
- serialNumber: string
- warrantyStatus: enum (valid, expired, void, non_registered)
- warrantyExpiryDate: ISO 8601 date
- originalPurchaseDate: ISO 8601 date
- manufacturerName: string
- registrationValid: boolean
- complianceCertification: string (reference number)

**Equipment Verification Request:**
- serialNumber: string (required)
- checkDate: ISO 8601 date (required)

**Equipment Verification Response:**
- valid: boolean
- warrantyActiveOn: boolean (for specific date)
- serviceNetwork: array (authorized service centers)

## Sample usage

```bash
# Verify ballot scanner warranty (requires API-key + reg number)
curl -X POST https://api.kebs.go.ke/warranty/lookup \
  -H "X-API-Key: $KEBS_WARRANTY_API_KEY" \
  -H "X-Manufacturer-Reg: $KEBS_MANUFACTURER_REG" \
  -H "Content-Type: application/json" \
  -d '{
    "serialNumber": "ES2024-001234",
    "productCategory": "ballot-scanner",
    "registrationNumber": "KBR-2025-0567"
  }'

# Check equipment warranty validity for election date
curl -X POST https://api.kebs.go.ke/warranty/verify \
  -H "X-API-Key: $KEBS_WARRANTY_API_KEY" \
  -H "X-Manufacturer-Reg: $KEBS_MANUFACTURER_REG" \
  -H "Content-Type: application/json" \
  -d '{
    "serialNumber": "ES2024-001234",
    "checkDate": "2025-08-09"
  }'

# Verify manufacturer is registered and compliant
curl -X GET "https://api.kebs.go.ke/manufacturer/status?reg=KBR-2025-0567" \
  -H "X-API-Key: $KEBS_WARRANTY_API_KEY"
```

## Health and readiness

```bash
curl -s -H "X-API-Key: $KEBS_WARRANTY_API_KEY" \
  https://api.kebs.go.ke/health
# Response: {"status": "ready", "database": "current"}
```

## Rate limits

100 requests per minute per API key

| Metric | Limit | Window |
|---|---|---|
| Warranty lookups | 100/min | rolling 1-minute |
| Manufacturer verifications | 50/min | rolling 1-minute |
| Bulk uploads | 10/hour | rolling hour |
| Daily requests | 50000/day | calendar day |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors (5xx, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `KEBS_WARRANTY_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 20s |
| error | return 503 Service Unavailable |
| invalid_serial | return not found error |

## Fallback strategy

Circuit open: use cached warranty database (refreshed weekly). For lookups not in cache, mark equipment status as "pending verification" and queue for manual KEBS review. Cache TTL: 7 days. Queue TTL: 14 days.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| KEBS_WARRANTY_BASE_URL | yes | - | KEBS API base URL |
| KEBS_WARRANTY_API_KEY | yes | - | API key for authentication |
| KEBS_MANUFACTURER_REG | yes | - | Manufacturer registration number |
| KEBS_TIMEOUT_SECS | no | 15 | Request timeout |
| KEBS_WARRANTY_CHAOS | no | false | Chaos mode (timeout/error/invalid_serial) |
| KEBS_CACHE_DB | no | ./lib/kebs-warranty.db | Offline warranty database |

## Related modules

- M66 (equipment-compliance) - primary consumer
- M77 (materials-warranty) - warranty tracking
- M83 (supply-chain-verify) - registration validator

## Runbook

Refer to `docs/runbooks/integrations/kebs-warranty.md` for API key rotation, manufacturer registration verification, and warranty database cache refresh procedures.
