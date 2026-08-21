# integrations/odpc-breach

REST data breach notification to Office of the Data Protection Commissioner (ODPC) with RSA-PSS signed messages.

**MoU Status: ODPC Data Breach Notification MoU signed**

## Module usage

- Data protection incident and breach notification reporting
- Compliance submission to data protection authority
- Breach investigation and mitigation tracking

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: mTLS (client certificate) + RSA-PSS request signing
Endpoints: ODPC breach notification API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/breach/report` | POST | Submit data breach notification |
| `/breach/{notificationId}` | GET | Get breach notification status |
| `/breach/{notificationId}/update` | PATCH | Update breach information |
| `/incidents/investigate` | POST | Report incident for investigation |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| affectedPersonCount | A (Restricted) | Number of impacted individuals |
| breachDescription | A (Restricted) | Nature and scope of breach |
| dataCategory | A (Restricted) | Type of personal data compromised |
| incidentDate | B (Internal) | When breach occurred |
| notificationDate | C (Public) | When notification was filed |

## Request and response shape

**Breach Notification Request:**
- organizationName: string (required)
- notificationType: enum (data_breach, security_incident, loss_of_confidentiality)
- affectedPersonCount: integer (required)
- dataCategory: array of strings (IDs, health, financial, voting_records)
- incidentDate: ISO 8601 date (required)
- discoveryDate: ISO 8601 date
- breachDescription: string (required, 100-5000 chars)
- causeFactor: enum (hacking, malware, human_error, insider, physical_theft)
- mitigationSteps: string

**Breach Notification Response:**
- notificationId: string (UUID)
- status: enum (received, under_review, acknowledged, action_required)
- referenceNumber: string (ODPC case reference)
- receivedDate: ISO 8601 timestamp
- acknowledgedDate: ISO 8601 timestamp (when applicable)
- comments: string (ODPC feedback)

## Sample usage

```bash
# Report data breach (requires mTLS + RSA-PSS signature)
curl -X POST https://api.odpc.go.ke/breach/report \
  --cert $ODPC_BREACH_CERT --key $ODPC_BREACH_KEY \
  -H "Content-Type: application/json" \
  -H "X-Signature: $(echo -n 'request_body' | openssl dgst -sha256 -sign $ODPC_BREACH_KEY | base64)" \
  -d '{
    "organizationName": "Electoral Commission",
    "notificationType": "data_breach",
    "affectedPersonCount": 5000,
    "dataCategory": ["voting_records", "personal_ids"],
    "incidentDate": "2025-08-10",
    "discoveryDate": "2025-08-11",
    "breachDescription": "Unauthorized access to voter database",
    "causeFactor": "hacking",
    "mitigationSteps": "Compromised systems taken offline, passwords reset"
  }'

# Check notification status
curl -X GET "https://api.odpc.go.ke/breach/NOTIF-2025-001" \
  --cert $ODPC_BREACH_CERT --key $ODPC_BREACH_KEY
```

## Health and readiness

```bash
curl -s https://api.odpc.go.ke/health
# Response: {"status": "ready", "version": "1.0"}
```

## Rate limits

10 requests per minute per mTLS certificate

| Metric | Limit | Window |
|---|---|---|
| Breach notifications | 10/min | rolling 1-minute |
| Status queries | 20/min | rolling 1-minute |
| Bulk submissions | 2/hour | rolling hour |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 2 consecutive errors (5xx, timeout, connection refused) |
| open duration | 60 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

## Chaos testing

Set `ODPC_BREACH_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 15s |
| error | return 503 Service Unavailable |
| slow | add 5s latency |

## Fallback strategy

Circuit open: queue breach notification to secure offline queue. Notifications sent to ODPC via registered email and secure courier with eel reference number. Manual follow-up by compliance team. Queue TTL: 7 days.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| ODPC_BREACH_BASE_URL | yes | - | ODPC API base URL |
| ODPC_BREACH_CERT | yes | - | mTLS client cert (PEM) |
| ODPC_BREACH_KEY | yes | - | mTLS private key (PEM) |
| ODPC_RSA_PSS_KEY | yes | - | RSA-PSS signing key |
| ODPC_BREACH_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| ODPC_BREACH_TIMEOUT_SECS | no | 15 | Request timeout |

## Related modules

- Data protection and compliance module - primary consumer
- Security incident response module - breach coordinator
- Audit and reporting module - ODPC liaison

## Runbook

Refer to `docs/runbooks/integrations/odpc-breach.md` for mTLS certificate rotation and RSA-PSS key management.
