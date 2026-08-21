# integrations/printer-mes

REST Manufacturing Execution System (MES) for ballot printer vendor coordination and production tracking.

**MoU Status: Ballot Printer Vendor NDA in place**

## Module usage

- Ballot printing job submission and status tracking
- Production schedule coordination with printer vendor
- Quality assurance and lot metadata management
- Print job authentication and audit trail

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: API-key
Endpoints: Printer vendor MES system

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/jobs/submit` | POST | Submit ballot printing job |
| `/jobs/{jobId}` | GET | Get job status and progress |
| `/lots/{lotId}` | GET | Get lot metadata and QA status |
| `/production/schedule` | GET | Get production timeline |
| `/delivery/track` | GET | Track delivery status |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| jobID | C (Public) | Print job identifier |
| lotCode | C (Public) | Production lot number |
| status | C (Public) | Job status (non-sensitive) |
| quantity | C (Public) | Number of ballots printed |
| deliveryDate | C (Public) | Expected delivery |
| qualityMetrics | C (Public) | QA pass rates (no PII) |

## Request and response shape

**Print Job Request:**
- jobName: string (required, descriptive)
- quantity: integer (required, number of ballots)
- specifications: object (paper type, ink, dimensions)
- deliveryDate: ISO 8601 date (required)
- priority: enum (normal, high, critical)
- specialInstructions: string (optional)

**Print Job Response:**
- jobID: string (UUID)
- lotCode: string (production batch identifier)
- status: enum (submitted, queued, printing, quality_check, packaged, ready_for_delivery)
- estimatedCompletion: ISO 8601 datetime
- statusUrl: string (tracking endpoint)

**Lot Status Response:**
- lotCode: string
- quantityProduced: integer
- quantityPassed: integer
- quantityRejected: integer
- passRate: float (0.0-1.0)
- qaSignoff: boolean
- signoffDate: ISO 8601 date (when applicable)

## Sample usage

```bash
# Submit printing job (requires API-key)
curl -X POST https://api.printer-vendor.com/jobs/submit \
  -H "X-API-Key: $PRINTER_MES_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "jobName": "August 2025 Election Ballots - Region 5",
    "quantity": 100000,
    "specifications": {
      "paperType": "security_paper",
      "dimensions": "210x297mm",
      "colors": 4
    },
    "deliveryDate": "2025-08-01",
    "priority": "high"
  }'

# Track job status
curl -X GET "https://api.printer-vendor.com/jobs/JOB-2025-001" \
  -H "X-API-Key: $PRINTER_MES_API_KEY"

# Get lot quality metrics
curl -X GET "https://api.printer-vendor.com/lots/LOT-2025-001" \
  -H "X-API-Key: $PRINTER_MES_API_KEY"
```

## Health and readiness

```bash
curl -s -H "X-API-Key: $PRINTER_MES_API_KEY" https://api.printer-vendor.com/health
# Response: {"status": "ready", "queue_depth": 12, "active_jobs": 3}
```

## Rate limits

30 requests per minute per API key

| Metric | Limit | Window |
|---|---|---|
| Job submissions | 30/min | rolling 1-minute |
| Status queries | 30/min | rolling 1-minute |
| Lot lookups | 20/min | rolling 1-minute |
| Bulk shipment queries | 5/hour | rolling hour |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 1 error (critical vendor system) |
| open duration | 120 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `PRINTER_MES_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: fall back to phone coordination with printer vendor. Fax or email job submission. Queue TTL: 24 hours. For status queries, use last known status and contact vendor hotline.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| PRINTER_MES_BASE_URL | yes | - | Printer vendor MES base URL |
| PRINTER_MES_API_KEY | yes | - | API key for authentication |
| PRINTER_MES_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| PRINTER_MES_TIMEOUT_SECS | no | 30 | Request timeout |

## Related modules

- Ballot preparation and printing module - primary consumer
- Materials supply chain module - vendor coordinator
- Election logistics module - delivery tracker

## Runbook

Refer to `docs/runbooks/integrations/printer-mes.md` for API key rotation and production schedule contingency procedures.
