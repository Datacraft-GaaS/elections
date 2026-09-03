# Personnel / Dispatch App (M33) — Completion Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/mobile-personnel-dispatch.md`. Backend: dispatch-api
routes already real (`/v1/dispatch/*` + `/api/v1/mobile/dispatch/*`); app
already integrates them. Est: 5 ED (completion + hardening).

## Current state

`services/field-mobile-svc/apps/personnel/` — 25 lib files, 6 test files,
real `DispatchApi` (dio), GPS, signing, workforce duty + notifications
screens. Gaps: placeholder biometric hash at check-in (documented deferral
D2), no attestation/MFA session, no outbox/offline write path, no push, no
substitution/status-ping durability, missing kiosk-free MFA pieces.

## Target features (completion)

| # | Feature | FR | Status |
|---|---|---|---|
| F1 | Enrolment + duplicate rejection | 001 | partial (workforce screens) |
| F2 | MFA: biometric + PIN fallback 5/30 min; token bound to attestation | 002 | missing |
| F3 | DPA consent screen (EN+SW) + withdraw | 003 | missing |
| F4 | Attestation tiers (soft-warn citizen, hard-fail supervisor) | 004 | missing |
| F5 | Assignment push (no PII) + detail screen cached offline | 005–006 | partial |
| F6 | Accept/decline with reason + substitution realtime | 007–008 | partial (duty) |
| F7 | Polygon GPS check-in signed with device key + biometric hash from paired kit | 009–011 | partial (placeholder hash) |
| F8 | Supervisor realtime dashboard feed (WebSocket) | 012 | backend + app |
| F9 | Status pings 15 min + missed-ping gap audit | 013 | partial |
| F10 | Explicit status screen (ON_STATION/BREAK/ABSENT + reasons) | 014 | partial |
| F11 | Check-out with materials return checklist | later FRs | partial |
| F12 | Offline queue for check-in/ping/status with UUID7 idempotency | FR set | missing (D2 blocker remains) |
| F13 | Force-update block + queue survival | 036-ish | missing |

## Files

```
lib/src/core/                     NEW: bootstrap/session/api_client(via kit-sdk)/local_db/outbox/tamper/config
lib/src/session.dart              REPLACE with eel-core session (OIDC + biometric + PIN lockout)
lib/src/services/dispatch_api.dart  keep; migrate onto EelHttpClient (pinning + idempotency interceptor)
lib/src/services/kit_pairing.dart NEW: BLE/mTLS pairing with BVI kit to obtain biometric hash (closes D2)
lib/src/services/polygon.dart     NEW: point-in-polygon + tolerance (or eel-core import)
lib/src/features/consent/         consent_screen (DPA), consent_repository
lib/src/features/check_in/        extend: polygon validation UI, kit-hash injection state, offline queueing
lib/src/features/status/          status_screen (ON_STATION/BREAK/ABSENT), ping_scheduler (15 min, foreground service)
lib/src/features/assignments/     accept_decline_sheet, substitution_listener (SSE)
lib/src/features/workforce/       existing duty/notifications: wire to push + outbox
lib/src/features/settings/        consent status, sync panel, version floor
test/... (extend), integration_test/..., golden/...
```

## Technical implementation

- **D2 closure (kit pairing)**: implement `KitPairingService`: scan for BVI
  kit BLE advertisement (`eel.kit` UUID), mutual auth via kit-sdk session
  (mTLS-over-BLE channel or signed challenge); on successful pairing, kit
  injects `biometric_hash` of the worker's enrolment template session.
  Until hardware pairing succeeds, app runs in `degraded` mode: check-in
  submits `biometric_hash: null` + `pairing_status: UNPAIRED` and backend
  marks check-in provisional (contract change documented in
  field-mobile-dispatch allowlist row note) — never a fabricated hash.
- **Polygon check-in**: fetch station polygon from dispatch-api; ray-casting
  test + tolerance (config); outside → refuse + supervisor alert
  (backend event); UI shows distance + direction.
- **Pings**: `Timer.periodic(15 min)` in foreground service with battery
  exemption requested; missed ping = local gap marker + outbox `gap_audit`;
  server reconciles last-seen.
- **Offline writes**: master §2.5 path; UUID7 ids; dispatch-api dedup via
  Idempotency-Key (add to dispatch-api if absent — 72 h store).
- **Substitutions**: SSE `/api/v1/mobile/bff/events` assignment channel;
  on REASSIGNED → local mark + notification.

## Algorithms

- Ray-casting point-in-polygon + haversine distance (shared with observer).
- Ping scheduler with Doze-aware WorkManager equivalent
  (`workmanager` plugin) + exact-allow foreground service during session.
- Provisional check-in state machine: `provisional → confirmed` on kit hash
  arrival (reconciled server-side by pairing event).

## Backend touchpoints

- dispatch-api: add Idempotency-Key store + polygon endpoints
  (`/v1/dispatch/stations/{code}/polygon`) if not present in spec; bind
  `field-mobile-dispatch` remaining unbound paths; provisional check-in
  contract note.

## UX / UI / ergonomics

- **Consent first**: on first launch, DPA screen (EN+SW) with two consents
  (location, attendance) as separate toggles + withdraw in Settings;
  blocking until location consent for field roles.
- **Home**: today's assignment card (station, report time, supervisor,
  contact buttons); big Check In button; status chip current state.
- **Check-in**: progress rail (GPS fix → kit pairing → sign → submit);
  outside-polygon state shows map + distance + "notify supervisor" action;
  provisional state explained in plain copy ("Kit not paired yet — check-in
  recorded as provisional").
- **Status**: 3 large buttons with reason sheets; BREAK requires expected
  return time; ABSENT requires reason; current status persistent chip.
- **Assignments**: accept/decline with mandatory decline reason; substitution
  banner when replaced (old marked REASSIGNED, no data loss).
- Sync panel (outbox) + stale-since; full state matrix; goldens.

## Testing

- Unit: polygon cases, ping scheduler (fake clock), provisional→confirmed
  FSM, lockout counter.
- Widget: consent, check-in rail states, status sheets, assignment decline.
- Integration: 250k-scale is backend; app-side: check-in → outbox →
  reconnect dedup; kill-drill at each state; pairing degraded path.
- Contract: Pact for dispatch paths used; idempotency tests on BFF.
- A11y: traversal of check-in + status; EN/SW + 9 locales complete.

## Definition of done

Master §9; D2 deferral row in `docs/plans/deferral-plan.md` updated to
[DONE] once pairing lands (or re-scoped with hardware blocker documented);
FRs tagged tests; ACs ticked.