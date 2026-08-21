# integrations/kmd

REST weather data and climate information from Kenya Meteorological Department (KMD).

**MoU Status: KMD partner agreement in place**

## Module usage

- Weather forecasting for election logistics and scheduling
- Climate data for field operations planning
- Environmental conditions monitoring

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: API-key
Endpoints: KMD weather API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/weather/forecast` | GET | Get weather forecast by coordinates |
| `/weather/current` | GET | Get current weather conditions |
| `/climate/historical` | GET | Get historical climate data |
| `/alerts/weather` | GET | Get active weather alerts |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| temperature | C (Public) | Weather data (no PII) |
| humidity | C (Public) | Meteorological data |
| rainfall | C (Public) | Climate information |
| coordinates | C (Public) | Location data (public) |
| timestamp | C (Public) | Observation time |

## Request and response shape

**Forecast Request:**
- latitude: float (required, -4.0 to 4.0)
- longitude: float (required, 28.0 to 42.0)
- days: integer (1-7, default 7)
- include_alerts: boolean

**Forecast Response:**
- forecastID: string (UUID)
- location: object (latitude, longitude, region)
- forecast: array of daily forecasts
- alerts: array (when applicable)
- generatedAt: ISO 8601 timestamp

## Sample usage

```bash
# Get weather forecast (requires API-key)
curl -X GET "https://api.kmd.go.ke/weather/forecast?latitude=-1.3&longitude=36.8&days=5" \
  -H "X-API-Key: $KMD_API_KEY"

# Get current conditions
curl -X GET "https://api.kmd.go.ke/weather/current?latitude=-1.3&longitude=36.8" \
  -H "X-API-Key: $KMD_API_KEY"
```

## Health and readiness

```bash
curl -s https://api.kmd.go.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

100 requests per minute per API key

| Metric | Limit | Window |
|---|---|---|
| Forecast requests | 100/min | rolling 1-minute |
| Bulk queries | 20/hour | rolling hour |

Cache TTL: 1 hour for forecast data

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors (5xx, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `KMD_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: use cached forecast data (up to 12 hours old). If no cache available, return last known good forecast. Operations team receives alert for manual weather monitoring.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| KMD_BASE_URL | yes | - | KMD API base URL |
| KMD_API_KEY | yes | - | API key for authentication |
| KMD_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| KMD_CACHE_TTL_SECS | no | 3600 | Cache TTL (1 hour) |
| KMD_TIMEOUT_SECS | no | 10 | Request timeout |

## Related modules

- Election logistics planning module - primary consumer
- Field operations module - weather-dependent scheduling

## Runbook

Refer to `docs/runbooks/integrations/kmd.md` for API key rotation and cache invalidation procedures.
