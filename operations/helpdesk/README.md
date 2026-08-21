# operations/helpdesk

Field support helpdesk integration for eel. Zammad OSS is the ticket backend. Tickets
are created automatically by `field-mobile-svc` (M32) and by officers via the support
mobile app. Ticket updates flow back to `support-admin-svc` via webhook.

## Overview

The helpdesk is the primary channel for field-level issues during registration and
election day: kit failures, connectivity problems, MDM enrolment failures, and data
integrity questions. The SLA matrix below governs response and resolution times.

## Ticket categories and SLAs

| Category | First response SLA | Resolution SLA | Escalation tier |
|----------|--------------------|----------------|-----------------|
| Kit hardware failure | 15 min | 4 hr | Field support lead |
| Kit software / attestation | 15 min | 2 hr | SRE on-call |
| Network / connectivity | 30 min | 4 hr | Network team |
| MDM enrolment failure | 15 min | 1 hr | SRE on-call |
| BVR/BVI data issue | 15 min | 1 hr | Data integrity team |
| RTS results issue | 10 min | 30 min | Election day ops lead |
| General inquiry | 2 hr | 8 hr | Helpdesk agent |

## SLA escalation matrix

| Breach level | Action |
|--------------|--------|
| First response SLA breached | Auto-escalate to Tier 2; notify field support lead |
| Resolution SLA at 50% | Notify assigned agent and team lead |
| Resolution SLA breached | Auto-escalate to SRE on-call; page via PagerDuty |
| P0 impact (data loss, results affected) | Invoke `docs/runbooks/incident-response-p0.md` immediately |

## Files

| File | Description |
|------|-------------|
| `zammad-config-template.yaml` | Zammad configuration: categories, priorities, SLAs, escalation trees |
| `integration-guide.md` | How `field-mobile-svc` creates tickets and how updates flow back |

## Usage

```bash
# Apply Zammad config via the Zammad REST API
# (requires ZAMMAD_API_TOKEN and ZAMMAD_URL from vault)
curl -H "Authorization: Token token=$ZAMMAD_API_TOKEN" \
  -H "Content-Type: application/json" \
  -X POST "$ZAMMAD_URL/api/v1/slas" \
  -d @<(yq -o json zammad-config-template.yaml | jq '.slas[0]')

# Verify SLA configuration is live
curl -H "Authorization: Token token=$ZAMMAD_API_TOKEN" \
  "$ZAMMAD_URL/api/v1/slas" | jq '.[].name'

# Create a test ticket (smoke test)
curl -H "Authorization: Token token=$ZAMMAD_API_TOKEN" \
  -H "Content-Type: application/json" \
  -X POST "$ZAMMAD_URL/api/v1/tickets" \
  -d '{"title":"Smoke test","group":"Field Support","article":{"body":"Test ticket","type":"note"}}'
```

## Config and environment

| Variable | Description |
|----------|-------------|
| `ZAMMAD_URL` | Zammad instance base URL |
| `ZAMMAD_API_TOKEN` | API token; retrieve from vault |
| `PAGERDUTY_ROUTING_KEY` | PagerDuty Events API v2 routing key for SRE escalation |

## Dependencies

- Zammad OSS >= 6.x
- `field-mobile-svc` (M32): ticket creation and status update API
- `support-admin-svc`: receives Zammad webhooks and surfaces to Commission console
- PagerDuty: SLA breach escalation

## Test and verify

```bash
# Check Zammad is reachable and authenticated
curl -s -H "Authorization: Token token=$ZAMMAD_API_TOKEN" \
  "$ZAMMAD_URL/api/v1/users/me" | jq '.login'

# Verify SLAs are configured
curl -s -H "Authorization: Token token=$ZAMMAD_API_TOKEN" \
  "$ZAMMAD_URL/api/v1/slas" | jq 'length'
# Expected: at least 7 (one per ticket category)
```

## Related docs

- `operations/README.md`: operations overview
- `operations/mdm/README.md`: MDM issues that generate tickets
- `operations/dispatch/README.md`: field dispatch triggered by helpdesk tickets
- `docs/runbooks/incident-response-p0.md`: P0 escalation runbook

## Limitations

- Zammad configuration is managed via the API; the YAML template is declarative intent,
  not directly applied. Apply each section via the corresponding API endpoint as shown.
- Automated SLA breach alerting to PagerDuty requires the Zammad PagerDuty trigger
  integration to be configured; see `integration-guide.md` for setup steps.
- Election-day ticket volume can exceed 1,000 per hour at peak; size the Zammad instance
  (Redis, PostgreSQL) accordingly before election day.
