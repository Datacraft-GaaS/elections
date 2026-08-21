# Helpdesk integration

## Overview

Field support tickets are created automatically by `field-mobile-svc` (M32 dispatch API) and routed to the helpdesk backend (Zammad OSS). Officers and party agents can also submit tickets via the support mobile app. Ticket updates flow back to `support-admin-svc` via webhook.

## Ticket categories

| Category | SLA: first response | SLA: resolution | Escalation tier |
|----------|---------------------|-----------------|-----------------|
| Kit hardware failure | 15 min | 4 hr | Field support lead |
| Kit software/attestation | 15 min | 2 hr | SRE on-call |
| Network/connectivity | 30 min | 4 hr | Network team |
| MDM enrolment failure | 15 min | 1 hr | SRE on-call |
| BVR/BVI data issue | 15 min | 1 hr | Data integrity team |
| RTS results issue | 10 min | 30 min | Election day ops lead |
| General inquiry | 2 hr | 8 hr | Helpdesk agent |

## SLA escalation matrix

| Breach level | Action |
|--------------|--------|
| First response SLA breached | Auto-escalate to tier 2 agent; notify field support lead |
| Resolution SLA at 50% | Notify assigned agent and team lead |
| Resolution SLA breached | Auto-escalate to SRE on-call; page via PagerDuty |
| P0 impact (data loss, results affected) | Immediately invoke `docs/runbooks/incident-response-p0.md` |

## Files

| File | Description |
|------|-------------|
| `zammad-config-template.yaml` | Zammad OSS configuration template: categories, priorities, SLAs, escalation trees |
| `integration-guide.md` | How `dispatch-api` creates tickets and how ticket updates flow back to `support-admin-svc` |

## Dependencies

- M32 `field-mobile-svc` dispatch API: creates and updates tickets via `integration-guide.md`.
- `support-admin-svc`: receives ticket webhooks and surfaces status to election commission console.
- PagerDuty: receives escalation triggers for SLA breaches on high-priority categories.
