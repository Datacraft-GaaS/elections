# integrations/pantone

Color palette matching and brand compliance validation for election materials.

**MoU Status: Pantone licensing agreement in effect**

## Module usage

- M67 (ballot-design) - ensure ballot colors meet accessibility standards
- M68 (candidate-branding) - validate candidate colors against brand guidelines
- M85 (materials-audit) - verify election materials color compliance

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: API-key
Endpoints: Pantone Color API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/match` | POST | Match RGB/HEX to nearest Pantone color |
| `/palette/generate` | POST | Generate accessible color palette |
| `/contrast/check` | POST | Check WCAG contrast compliance |
| `/validation/accessibility` | POST | Full accessibility color validation |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| colorValue | C (Public) | RGB/HEX color code (non-sensitive) |
| pantoneID | C (Public) | Pantone color reference (public standard) |
| complianceStatus | C (Public) | Accessibility check results |
| contrastRatio | C (Public) | WCAG contrast metrics |

## Request and response shape

**Color Match Request:**
- colorValue: string (required, HEX or RGB format)
- format: string (required, enum: hex, rgb, cmyk)
- tolerance: integer (0-100, default: 5)

**Color Match Response:**
- pantoneID: string (e.g., "18-0605")
- pantoneString: string (e.g., "Black Coffee")
- deltaE: number (color difference metric)
- wcagLevel: string (enum: AA, AAA, fail)

**Palette Generate Request:**
- baseColor: string (HEX format)
- paletteSize: integer (3-8)
- accessible: boolean (enforce WCAG AA)

**Palette Generate Response:**
- palette: array [{pantoneID, hex, name, wcagLevel}]
- complianceScore: number (0.0-1.0)

## Sample usage

```bash
# Match color to Pantone (requires API-key)
curl -X POST https://api.pantone.com/match \
  -H "X-API-Key: $PANTONE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "colorValue": "#FF0000",
    "format": "hex",
    "tolerance": 5
  }'

# Generate accessible palette
curl -X POST https://api.pantone.com/palette/generate \
  -H "X-API-Key: $PANTONE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "baseColor": "#003366",
    "paletteSize": 5,
    "accessible": true
  }'

# Validate contrast for ballot design
curl -X POST https://api.pantone.com/contrast/check \
  -H "X-API-Key: $PANTONE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "foregroundColor": "#000000",
    "backgroundColor": "#FFFFFF",
    "wcagTarget": "AA"
  }'
```

## Health and readiness

```bash
curl -s -H "X-API-Key: $PANTONE_API_KEY" https://api.pantone.com/health
# Response: {"status": "ready", "version": "2.5"}
```

## Rate limits

100 requests per minute per API key

| Metric | Limit | Window |
|---|---|---|
| Color matches | 100/min | rolling 1-minute |
| Palette generations | 50/min | rolling 1-minute |
| Validation requests | 100/min | rolling 1-minute |
| Daily API usage | 50000/day | calendar day |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors (5xx, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `PANTONE_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 20s |
| error | return 503 Service Unavailable |
| invalid_match | return no match found error |

## Fallback strategy

Circuit open: use offline lookup table embedded in application. Pantone color database cached at build time (covers 3000+ standard colors). For contrast validation, apply conservative WCAG AAA standards locally. Queue fallback requests for manual review. Queue TTL: 24 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| PANTONE_BASE_URL | yes | - | Pantone API base URL |
| PANTONE_API_KEY | yes | - | API key for authentication |
| PANTONE_TIMEOUT_SECS | no | 15 | Request timeout |
| PANTONE_CHAOS | no | false | Chaos mode (timeout/error/invalid_match) |
| PANTONE_OFFLINE_DB | no | ./lib/pantone.db | Offline color database path |

## Related modules

- M67 (ballot-design) - primary consumer
- M68 (candidate-branding) - validates brand colors
- M85 (materials-audit) - compliance verification

## Runbook

Refer to `docs/runbooks/integrations/pantone.md` for offline database updates and WCAG compliance troubleshooting.
