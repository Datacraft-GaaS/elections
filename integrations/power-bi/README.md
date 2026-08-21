# integrations/power-bi

Power BI Premium REST API for election analytics reports and real-time dashboards.

**MoU Status: Microsoft Enterprise Agreement (EA) in place**

## Module usage

- Election analytics dashboards and real-time reporting
- Candidate performance metrics and polling data visualization
- Electoral commission oversight and audit reporting

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: OAuth2 Azure AD (tenant-based service principal)
Endpoints: Power BI REST API (Premium capacity)

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/groups/{groupId}/reports` | GET | List reports in workspace |
| `/reports/{reportId}` | GET | Get report metadata |
| `/reports/{reportId}/pages` | GET | List report pages |
| `/dashboards` | GET | List dashboards |
| `/datasets/refresh` | POST | Trigger dataset refresh |
| `/admin/datasets` | GET | Admin API - list all datasets |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| reportName | C (Public) | Report title (public dashboards) |
| datasetName | C (Public) | Dataset identifier |
| refreshTime | C (Public) | Data refresh timestamp |
| workspaceID | C (Public) | Workspace reference |
| aggregatedMetrics | C (Public) | Summary stats (non-identifying) |

## Request and response shape

**Report Query Request:**
- groupId: string (required, Power BI workspace ID)
- reportId: string (required)
- filters: array of objects (optional, DAX filter expressions)
- pageNumber: integer (optional)

**Report Query Response:**
- id: string (UUID)
- name: string (report name)
- webUrl: string (embedded report URL)
- pages: array [{name, displayName}]
- datasetId: string (backing dataset)
- createdDate: ISO 8601 timestamp
- modifiedDate: ISO 8601 timestamp

## Sample usage

```bash
# Get reports from workspace (requires Azure AD OAuth2)
curl -X GET "https://api.powerbi.com/v1.0/myorg/groups/GROUP-ID/reports" \
  -H "Authorization: Bearer $POWER_BI_TOKEN"

# Refresh dataset
curl -X POST "https://api.powerbi.com/v1.0/myorg/datasets/DATASET-ID/refreshes" \
  -H "Authorization: Bearer $POWER_BI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{}'
```

## Health and readiness

```bash
curl -s -H "Authorization: Bearer $POWER_BI_TOKEN" https://api.powerbi.com/v1.0/myorg
# Response: {"version": "2.0"}
```

## Rate limits

Power BI Premium capacity model:

| Metric | Limit | Window |
|---|---|---|
| API calls | 200/hour | rolling hour (Premium) |
| Dataset refreshes | 8/day | per dataset |
| Report renders | 1000/hour | per capacity |

Rate limit headers (X-PowerBI-Retry-After) returned in responses.

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors (5xx, 429, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `POWER_BI_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: use cached report data (TTL: 1 hour). Automated daily snapshots of key dashboards. For real-time metrics, fall back to secondary analytics tool. Queue TTL: 6 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| POWER_BI_TENANT_ID | yes | - | Azure AD tenant ID |
| POWER_BI_CLIENT_ID | yes | - | OAuth2 service principal client ID |
| POWER_BI_CLIENT_SECRET | yes | - | OAuth2 service principal secret |
| POWER_BI_WORKSPACE_ID | yes | - | Power BI Premium workspace ID |
| POWER_BI_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| POWER_BI_TIMEOUT_SECS | no | 30 | Request timeout |
| POWER_BI_CACHE_TTL_SECS | no | 3600 | Cache TTL (1 hour) |

## Related modules

- Election analytics module - primary consumer
- Reporting and audit module - dashboard provider
- Data warehouse integration - ETL target

## Runbook

Refer to `docs/runbooks/integrations/power-bi.md` for Azure AD credential rotation and dataset refresh troubleshooting.
