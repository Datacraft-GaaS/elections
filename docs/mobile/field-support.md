# Field Support App (M32) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/mobile-field-support.md` (36 FRs). Contract to author:
`packages/openapi-contracts/field/field-support.yaml` (10 paths). Backend
crate with real routes already exists:
`services/field-mobile-svc/crates/support-api` (`/v1/support/*`). Est: 8 ED.

## Current state

`services/field-mobile-svc/apps/field-support/` — 12 lib files. All 4 screens
static text; `TicketService` uses bare `Dio()` with no baseUrl and matches no
real route; no auth; no offline.

## Target features

| # | Feature | FR |
|---|---|---|
| F1 | Enrolment + cluster assignment + MFA (biometric+PIN+attestation) | 001–003 |
| F2 | Live ticket queue (cluster/severity/age filters, ≤ 2 s refresh) | 004 |
| F3 | SLA timers (SEV1 2 h/1 d, SEV2 4 h/2 d, SEV3 6 h; amber 80 %, red 100 %) | 005 |
| F4 | Take/assign/re-assign with dual push + history reason | 006 |
| F5 | Threaded messaging + attachments (photo/video ≤ 60 s/log) | 007 |
| F6 | Kit lookup by serial/QR across cluster; cross-cluster requires reason + audit | 009 |
| F7 | Kit health telemetry panel (battery, connectivity, GPS, temp, errors, last-seen) | 010 |
| F8 | Remote diagnostics (logs/reboot/attest/self-test) — policy engine + kit consent + biometric-signed invocation + PO notification | 011–012 |
| F9 | Warranty check + claim filing + status push (O04) | 013–015 |
| F10 | Spares availability + dispatch (M20) + nearest store map (offline tiles) | 016, 026 |
| F11 | Escalation matrix L2/L3/Mechrevo/IEBC ICT with full context carry | 017–019 |
| F12 | On-call rotation view | 020 |
| F13 | Shift handoff (dual sign-off) + auto shift report (signed) | 022, 027–028 |
| F14 | Offline runbook library (symptom/error-code search ≤ 2 s) + reset guidance | 023, 025, 029 |
| F15 | Offline queue snapshot + stale-since; writes require connectivity; queued remote actions idempotent | 030–032 |
| F16 | Security envelope + battery budgets | 033–036 |

## Files

```
lib/main.dart                       rewrite bootstrap
lib/src/core/*                      eel-core (session/api/db/outbox/config/tamper/connectivity)
lib/src/features/auth/              login + cluster assignment display
lib/src/features/tickets/           ticket_queue_screen, ticket_detail, sla_timer (EelSlaTimerChip),
                                    take_assign_controller, thread_screen, evidence_upload
lib/src/features/kit/               kit_lookup_screen (serial+QR), kit_health_panel, remote_actions_sheet
lib/src/features/warranty/          warranty_screen, claim_flow
lib/src/features/spares/            spares_screen, store_map (offline tiles)
lib/src/features/escalation/        escalation_sheet (matrix + context bundle)
lib/src/features/shift/             shift_handoff_screen, shift_report_screen, oncall_view
lib/src/features/runbooks/          runbook_library, search_index (FTS5), recovery_guide
lib/src/features/contacts/          contact_card
lib/src/services/ticket_service.dart  REPLACE with repository over EelHttpClient + support-api paths
test/..., integration_test/..., golden/...
```

## Technical implementation

- **Ticket lifecycle client**: repository mirrors support-api routes
  (`/v1/support/tickets`, `{id}/take`, `{id}/escalate`, kits, action,
  warranty, spares, oncall, handoff, shift-report) — note support-api also
  serves `/api/v1/mobile/support/*` spec routes; use those for the app
  (contract-first field-support.yaml will formalize; keep one canonical set —
  the `/api/v1/mobile/support/*` set — and mark `/v1/*` internal).
- **SLA math**: `SlaTimer` computes fraction elapsed from
  `created_at + target`; thresholds 0.8/1.0; timer chip rebuilds each second
  only when visible (perf); persisted so app restart keeps state (EC-MOB-01);
  breach pushes come from backend; local timer is display-only.
- **Remote diagnostics**: action sheet lists actions from policy engine
  (config: allowed actions × time window); each invocation = biometric prompt
  → Ed25519 sign action envelope → `POST /v1/support/kits/{serial}/action`
  with UUID7; kit-side consent is a backend-mediated async flow — UI shows
  consent states (pending/granted/denied/timeout) with poll + push;
  unreachable kit → outbox queued action (FR-031).
- **Evidence**: attachments hashed (SHA-256) and uploaded to MinIO via BFF
  presigned flow; offline → blob in Isar + outbox entry; size caps enforced
  client-side.
- **Runbook search**: FTS5 index over signed bundle (same machinery as
  poll-clerk F2); index keyed by symptom keywords + kit error codes; version
  chip displayed.
- **Escalation context bundle**: serialized ticket JSON (description,
  attachments manifest, thread, telemetry snapshot with timestamp) attached
  to escalation POST; receiver app opens it read-only.

## Algorithms

- SLA fraction + color thresholds; amber/red transitions hysteresis-free.
- Policy engine: simple rule table from config `{action, role, window,
  severity_min}` evaluated client-side for UX, enforced server-side.
- QR/serial normalization + cluster scope check before request.
- Full-jitter backoff for queued actions; idempotency keys per action.

## Backend touchpoints

- `field-support.yaml` authored + bound; support-api handlers exist — add
  idempotency store + policy evaluation + kit-gateway proxy for actions;
  warranty/spares adapters already listed in module integrations.
- Events emitted (`eel.field.support.v1`): ticket_escalated (exactly-once),
  diagnostic_action, warranty_claim, shift_report.

## UX / UI / ergonomics

- **Tabs**: Tickets · Kits · Shift · More.
- **Queue**: filter chips (cluster, severity, age) as a horizontal row; rows
  show SLA chip (live), kit serial, station, assignee avatar; swipe actions:
  take (primary), escalate (secondary); pull-to-refresh; stale-since bar when
  offline snapshot.
- **Ticket detail**: header with SLA ring; action bar (Take/Start/Escalate/
  Resolve); thread below; evidence gallery; re-assign sheet requires reason.
- **Kit panel**: telemetry as 6 stat tiles with timestamps + sparkline for
  temperature/errors; remote actions in bottom sheet with consent states as
  stepper; every action shows "signed by you at HH:MM".
- **Shift**: handoff = 2-screen dual signature flow with open-ticket
  checklist (each ticket explicitly transferred); report preview with
  per-section provenance; on-call card with next handoff countdown.
- **Runbooks**: search-first page; results with highlighted snippet; reader
  with step checkboxes + links to recovery guide; version chip.
- Colour is never sole signal: SLA chips carry label (RESPONSE/RESOLUTION +
  % elapsed) per FR-004 a11y requirement; severity badges have text.
- All states per master §5.3 incl. `permission-denied` for remote actions
  outside policy window.

## Testing

- Unit: SLA thresholds/boundaries, policy engine matrix, shift-report
  aggregation, escalation context bundle round-trip, search index.
- Widget: queue with mixed SLA states, action sheet states, handoff flow.
- Integration: take → message → attach → escalate (context intact); queued
  remote action after kill; reconnect dedup; offline queue snapshot read.
- Contract: Pact for all 10 paths; exactly-once escalation event test.
- A11y: SLA timer announces minutes remaining (Semantics), traversal of
  ticket flow; locales complete (en/sw + 9).

## Definition of done

Master §9; FRs 001–036; ACs in module spec ticked; TicketService replaced;
contract bound; MDM staging install verified.