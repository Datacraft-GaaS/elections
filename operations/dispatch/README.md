# operations/dispatch

Field support dispatch and surge procedures for eel election day operations. Covers the
communications tree for coordinating field support teams, the surge procedure for deploying
additional technicians when incident volume exceeds normal capacity, and the dispatch API
integration with `field-mobile-svc`.

## Purpose

On election day, 46,229 polling stations operate simultaneously. Field support teams are
pre-positioned at county hubs. When a kit fails or a polling station has a connectivity
issue, the dispatch workflow routes the right technician to the right location within the
helpdesk SLA window (15-30 minutes for kit failures).

## Contents

| File | Description |
|------|-------------|
| `comms-tree.md` | Communications tree: who calls whom at each incident severity |
| `surge-procedure.md` | Surge procedure: how to activate additional field support capacity |
| `README.md` | This file |

## Dispatch flow

1. Officer reports issue via support mobile app or helpdesk phone line.
2. `field-mobile-svc` creates a helpdesk ticket and determines dispatch priority.
3. Dispatch console (in `support-admin-svc`) assigns nearest available technician.
4. Technician receives job on their support mobile app with GPS routing.
5. On arrival and resolution, technician closes ticket via mobile app.
6. `inventory-svc` records kit replacement or repair event if hardware was swapped.

```bash
# List pending dispatch jobs (dispatch admin API)
curl -H "Authorization: Bearer $DISPATCH_TOKEN" \
  https://dispatch.eel.internal/api/v1/jobs?status=pending \
  | jq '.[].priority'

# Assign a job to a technician
curl -X PATCH \
  -H "Authorization: Bearer $DISPATCH_TOKEN" \
  -H "Content-Type: application/json" \
  https://dispatch.eel.internal/api/v1/jobs/<job-id> \
  -d '{"technician_id":"<uuid>","status":"assigned"}'
```

## Surge procedure

See `surge-procedure.md` for the full procedure. Surge is activated when:

- Ticket queue exceeds 200 open tickets nationally, or
- Average time-to-assign exceeds 10 minutes for kit failure tickets, or
- A county has more than 5% of polling stations reporting active issues simultaneously.

Surge activates the on-call reserve technician pool and opens the emergency comms bridge
(documented in `comms-tree.md`).

## Communications tree

See `comms-tree.md`. Summary:

| Severity | Comms channel | Response lead |
|----------|---------------|---------------|
| Normal | Helpdesk ticket | Assigned technician |
| Elevated (county > 2% affected) | County WhatsApp group | County field lead |
| Surge (national > 200 open) | Emergency bridge call | National operations lead |
| P0 (results at risk) | Direct call chain | Elections director + SRE lead |

## Config and environment

| Variable | Description |
|----------|-------------|
| `DISPATCH_TOKEN` | Admin token for dispatch API; from vault |
| `DISPATCH_URL` | Base URL for `field-mobile-svc` dispatch API |

## Dependencies

- `field-mobile-svc` (M32): creates tickets and exposes the dispatch job API
- `support-admin-svc`: Commission console showing dispatch status
- `operations/helpdesk/`: SLA thresholds that trigger dispatch priority

## Test and verify

```bash
# Verify dispatch API health
curl https://dispatch.eel.internal/healthz

# Create a test dispatch job
curl -X POST \
  -H "Authorization: Bearer $DISPATCH_TOKEN" \
  -H "Content-Type: application/json" \
  https://dispatch.eel.internal/api/v1/jobs \
  -d '{"ticket_id":"<uuid>","priority":"P2","location":{"lat":-1.28,"lng":36.82}}'
```

## Related docs

- `operations/README.md`: operations overview
- `operations/helpdesk/README.md`: SLA thresholds that drive dispatch priority
- `operations/warranty/README.md`: spare kit stock for dispatch replacement
- `docs/runbooks/incident-response-p0.md`: P0 escalation from dispatch

## Limitations

- Dispatch GPS routing depends on technicians having data connectivity; in low-connectivity
  areas, coordinate by phone using the `comms-tree.md` procedures.
- The surge procedure requires pre-registered reserve technicians in the dispatch system;
  confirm the reserve roster is populated at least 72 hours before election day.
- Dispatch job assignment is manual in the current implementation; automated nearest-
  technician routing is a future enhancement.
