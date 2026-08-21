# Helpdesk integration guide

## Overview

`field-mobile-svc` (the M32 dispatch API) creates Zammad tickets via the Zammad REST API when a field officer reports an issue. Ticket updates (status changes, comments, escalations) flow back to `support-admin-svc` via a Zammad outbound webhook.

## Ticket creation flow

1. Field officer submits an issue via the support mobile app or the `field-mobile-svc` dispatch endpoint.
2. `field-mobile-svc` maps the issue type to a Zammad category and calls the Zammad API.
3. Zammad assigns the ticket to the correct group and starts the SLA clock.
4. `field-mobile-svc` stores the Zammad ticket ID in its local database and returns it to the mobile client.

```bash
# example: create a ticket from dispatch-api
curl -sf -X POST https://helpdesk.eel.io/api/v1/tickets \
  -u "<ZAMMAD_API_USER>:<ZAMMAD_API_TOKEN>" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"Kit hardware failure at station <STATION_ID>\",
    \"group\": \"Field Support\",
    \"customer\": \"<OFFICER_EMAIL>\",
    \"article\": {
      \"subject\": \"Kit hardware failure\",
      \"body\": \"Station: <STATION_ID>\\nKit serial: <SERIAL>\\nSymptom: <DESCRIPTION>\",
      \"type\": \"note\",
      \"internal\": false
    },
    \"state\": \"new\",
    \"priority\": \"2 high\",
    \"tags\": [\"kit-hardware\", \"station:<STATION_ID>\"]
  }"
```

## Ticket update flow (Zammad to support-admin-svc)

Zammad calls the outbound webhook on every ticket event. `support-admin-svc` receives the payload and updates its internal ticket mirror.

```bash
# example webhook payload from Zammad (simplified):
# POST http://support-admin-svc.eel-system.svc.cluster.local:8080/helpdesk/webhook
# {
#   "ticket": {"id": 1234, "number": "1234", "state": "open", "priority": "2 high"},
#   "event": "ticket.updated",
#   "changes": {"state": {"from": "new", "to": "open"}}
# }

# support-admin-svc verifies the webhook secret header (X-Zammad-Token)
# and emits a CloudEvent to Kafka:
# event_type: helpdesk.ticket.updated.v1
```

## Verification

```bash
# confirm Zammad is reachable from the cluster:
kubectl exec -n eel-system deploy/field-mobile-svc -- \
  curl -sf https://helpdesk.eel.io/api/v1/users/me \
  -u "<ZAMMAD_API_USER>:<ZAMMAD_API_TOKEN>" | grep '"login"'

# confirm webhook delivery (check Zammad webhook log in admin UI):
# Admin -> Integrations -> HTTP Callbacks -> recent deliveries

# confirm support-admin-svc receives events:
kubectl logs -n eel-system deploy/support-admin-svc --tail=20 | grep "helpdesk.ticket"
```

## Configuration

The integration uses these environment variables in `field-mobile-svc`:

| Variable | Description |
|----------|-------------|
| `ZAMMAD_BASE_URL` | Zammad instance URL (e.g., `https://helpdesk.eel.io`) |
| `ZAMMAD_API_TOKEN` | API token from Zammad admin settings |
| `ZAMMAD_WEBHOOK_SECRET` | Shared secret for webhook verification |

These are stored in OpenBao at `kv/eel/prod/helpdesk/zammad`.

## Error handling

If the Zammad API is unreachable, `field-mobile-svc` queues the ticket creation request in its local outbox and retries with exponential backoff. The officer receives a local confirmation number until the Zammad ticket is created and the IDs are reconciled.
