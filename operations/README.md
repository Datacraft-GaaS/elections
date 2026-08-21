# operations

Field and back-office operations tooling for the eel election platform. Covers the full
lifecycle of the 65,000+ KIEMS devices and the humans who use them: device management,
helpdesk ticketing, e-learning, warranty and RMA, dispatch and field surge, and legacy
data migration from the previous KIEMS system.

## Contents

| Directory | Description |
|-----------|-------------|
| `mdm/` | Mobile Device Management: Android Enterprise (BVI/BVR/RTS kits) and Apple Business Manager (staff iOS) |
| `helpdesk/` | Zammad OSS integration for field support tickets; SLA escalation matrix |
| `e-learning/` | LMS content and approval workflow for polling officer and party agent training |
| `warranty/` | Device warranty tracking, coverage matrix, RMA workflow |
| `dispatch/` | Field support dispatch: surge procedures, communications tree |
| `data-migration/` | One-way migration of voter and results data from legacy KIEMS to eel services |

## Scope in the wave plan

Operations tooling spans Wave 4 and Wave 5:

- WP-4.x: MDM profiles, helpdesk integration, e-learning LMS
- WP-5.x: Data migration, warranty tracking, full field dispatch procedures

## Related services

| Service | Operations touch point |
|---------|----------------------|
| `inventory-svc` (M20) | Device custody events at every MDM lifecycle stage |
| `field-mobile-svc` (M32) | Creates helpdesk tickets; triggers dispatch |
| `support-admin-svc` | Receives helpdesk webhooks; election commission console |
| `e-learning-svc` | Serves LMS content; tracks officer completion |
| `bvr-svc`, `bvi-svc`, `rts-svc` | Targets for legacy data migration |

## Quick navigation

- Device policy: `operations/mdm/README.md`
- Kit replacement SLA: `operations/helpdesk/README.md` (Kit hardware failure row)
- Officer training: `operations/e-learning/README.md`
- RMA process: `operations/warranty/rma-workflow.md`
- Field surge: `operations/dispatch/surge-procedure.md`
- Data migration order of operations: `operations/data-migration/migration-runbook.md`

## Election-day operations flow

On election day, the following sequence applies:

1. 72 hours before: confirm spare kit stock per county (`operations/warranty/`); confirm dispatch roster is populated.
2. 24 hours before: apply election-day MDM profile push; verify all officer e-learning completions.
3. T-2hr: ops center opens; dispatch console live; helpdesk staffed at surge capacity.
4. Polls open: kit heartbeat monitoring active; helpdesk SLA matrix enforced.
5. Polls close: results transmission window; RTS helpdesk SLA tightens to 10min/30min.
6. Post-election: petition window begins; KIEMS stays read-only; device wipe/decommission after 60 days.

## Runbook index

| Situation | Runbook |
|-----------|---------|
| Kit fails during voting | `operations/helpdesk/README.md`, then `operations/dispatch/surge-procedure.md` |
| Connectivity outage at polling station | `operations/helpdesk/README.md` (network category) |
| MDM enrolment failure | `operations/mdm/README.md`, remote command catalog |
| Results data issue | `operations/data-migration/README.md` + P0 runbook |
| P0 (data loss, results at risk) | `docs/runbooks/incident-response-p0.md` |

## Related docs

- `docs/runbooks/incident-response-p0.md`: P0 incident escalation (referenced by helpdesk)
- `docs/plans/00-principles.md`: wave overview (WP-4.x, WP-5.x)
- `services/inventory-svc/`: device inventory service
