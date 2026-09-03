# Poll Clerk App (M31) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec: `docs/plans/modules/mobile-poll-clerk.md`
(32 FRs, 15 ACs). Upstream contract to create:
`packages/openapi-contracts/field/poll-clerk.yaml`. BFF: mobile-bff §4.1.
Estimate: 12 ED.

## Current state

`services/field-mobile-svc/apps/poll-clerk/` — 27 lib files, 1 trivial widget
test, 11 locales present. Auth is a fake 300 ms delay; `AttestationService`
always passes with `stub-token` (ADR-0021 violation); `LocalDbService` no-op;
`ApiService` unused by any screen; 4 of 5 screens are `Text('... stub')`.

## Target features (map to FRs)

| # | Feature | FR(s) |
|---|---|---|
| F1 | Bootstrap + hard attestation + MFA sign-in (biometric + PIN + device attestation; 5-fail lockout 30 min) | 001–003 |
| F2 | Reference library: polling manual (EN+SW), forms library (34A/B/C PDF export), Elections Act clauses, contact card — all offline, full-text search ≤ 1 s | 004–007, 025 |
| F3 | Station dashboard mirror (BVI back-end): turnout, auth count, kit battery, connectivity, GPS fix; ≤ 30 s refresh; read-only; WebSocket | 008–009 |
| F4 | Voter register lookup fallback: DPO biometric attestation + written justification; rate-limited per station/hour from policy bundle; permanent audit; offline → queued pending with info status | 010–012, 027 |
| F5 | Queue management: register presence, call-next, priority sub-queue (elderly/pregnant/disabled), numeric tickets, live position + ETA, metrics panel | 013–015 |
| F6 | Structured incident form: 7 categories, free text, photo/video evidence with size caps, escalation chain (PO→RO→county→war-room, immutable level, signed timestamps) | 016–018 |
| F7 | Paper-backup workflow: step wizard for KIEMS-down (paper auth steps, Form 34A guide, spoilt-ballot serial log, digital reconciliation checklist) | 019–020 |
| F8 | Kiosk mode (KIOSK_OPERATOR): BVV register lookup with staff-assisted OTP; objection filing with citizen oral consent + witness biometric + witness signature (DPA 2019 record) | 021–022 |
| F9 | End-of-day report: auto-compiled from local records; dual biometric signature (PO + DPO); UUID7 idempotent submit; queued offline | 023–024, 026 |
| F10 | Security envelope: mTLS pinning, captive-portal UX, FLAG_SECURE on lookup+incident screens, SQLCipher/Isar with hardware key, tamper events, RASP | 029–030, 032 |
| F11 | Battery discipline: ≤ 5 %/h fg; bg sync ≤ 2 %/h; Doze respected; fg-service only for queue polling | 028 |

## Files to create/modify

```
lib/main.dart                          rewrite: bootstrap via eel-core-flutter
lib/src/app_router.dart                guard redirects; kiosk route by role
lib/src/core/{bootstrap,session,api_client,local_db,outbox,connectivity,app_config,tamper}.dart
lib/src/features/auth/                 login_screen (MFA UI), session_controller
lib/src/features/dashboard/            station_dashboard_* (real, WebSocket)
lib/src/features/lookup/               lookup_screen, lookup_controller, rate-limit gate, justification sheet
lib/src/features/queue/                queue_screen, ticket_model, priority_engine, metrics_panel
lib/src/features/incident/             incident_screen (7 cats), evidence_upload, escalation
lib/src/features/paper_backup/         wizard_screen, checklist_model, reconciliation
lib/src/features/kiosk/                kiosk_screen, otp_assist, objection_flow (consent + witness)
lib/src/features/eod_report/           eod_screen, compiler.dart (aggregation), dual_signature_sheet
lib/src/features/reference/            manual_screen, forms_screen, clauses_screen, contacts_screen, search_index.dart
lib/src/services/attestation_service.dart  DELETE (replaced by kit-sdk)
lib/src/services/local_db_service.dart     DELETE (replaced by core/local_db)
l10n/*.arb                             extend all 11 locales
test/...  integration_test/...  test/golden/...
```

## Technical implementation

- **F1 session**: eel-core `SessionController`: OIDC PKCE (flutter_appauth) →
  app JWT via BFF `/sessions`; attestation verdict required in exchange;
  biometric gate (`local_auth`) on foreground-resume; PIN fallback with
  monotonic lockout persisted in secure store. SpiceDB role from token decides
  routes (DPO vs CLERK vs KIOSK).
- **F2 offline content**: content bundles as versioned, ed25519-signed archives
  in `assets/content/` (manifest: version, sha256, signature, per-file hashes);
  loaded into an SQLite FTS5 index (package `sqlite3`) built at first launch
  (WAL mode); search uses BM25; PDF export via `pdf`/`printing` package with
  pinned template version. Update only via signed MDM push (verify signature
  before swap; keep N-1). AC-M31-05 benchmark test against fixture index.
- **F3 dashboard**: WebSocket (BFF SSE `/api/v1/mobile/bff/events` filtered by
  station) with 30 s poll fallback; strictly read-only rendering; errors =
  informational banner (FR-009); last-good cache displayed with stale-since.
- **F4 lookup**: UI forces justification textarea + DPO biometric prompt
  before request; client-side rate-limit pre-check from policy bundle
  (token bucket per station-hour, refetch with config); server is authority
  (429 → `rate-limited` state with reset time); offline → outbox
  `lookup_request` type with `pending-info` status; on completion push →
  banner "lookup result ready".
- **F5 queue**: `PriorityEngine` — two-tier comparator: priority flag first,
  then ticket number; ETA = rolling average service time (EWMA α=0.3) ×
  position; metrics from local counters + server snapshot. Call-next requires
  PO role; promotions audit-logged locally + outboxed.
- **F6 incidents**: `IncidentCategory` enum (7 values); evidence pipeline:
  image_picker/video ≤ 60 s; per-file 25 MB cap; SHA-256 + store in Isar blob;
  upload to BFF `POST /api/v1/mobile/bff/incidents` with multipart
  (`Idempotency-Key: <incident uuid7>`); escalation FSM: level immutable once
  set > 0 (DB CHECK + UI lock); signature = Ed25519 over canonical JSON
  (kit-sdk `EelSigningKey`).
- **F7 paper-backup**: `EelStepFlow` wizard (7 steps), each step = checklist
  item with optional serial-number input; spoilt-ballot log = append-only
  table; reconciliation checklist creates an outbox item on completion.
- **F8 kiosk**: role-gated route; OTP assist = staff fetches OTP for citizen
  (BVV adapter via BFF); objection flow = 3-screen consent capture: citizen
  oral consent checkbox + recording of consent metadata, witness biometric
  (`local_auth`), witness signature (signature pad → vector hash, raw
  discarded per kit pattern); DPA lawful-basis screen shown first; all stored
  locally then outboxed to M07 path.
- **F9 EOD**: `EodCompiler` reads local tables (check-in, incidents,
  spoilt log, agent presence) → draft; dual signature: PO signs, DPO signs
  (two biometric prompts, both Ed25519 signatures recorded); submit with
  UUID7; queued offline; screen blocks session close until confirmed.
- **F10**: per master §3; FLAG_SECURE via `EelSecureScreen` on lookup +
  incident screens.

## Algorithms

- Token-bucket rate limiter (station, hour): capacity = policy bundle limit;
  refill continuous; display remaining + reset.
- EWMA service-time ETA (α = 0.3, seeded 90 s).
- BM25 search over FTS5; snippet extraction; ICU normalization (NFC) before
  index; Unicode corpus test (EC-DAT-09).
- Canonical JSON hashing (RFC 8785 semantics) for signature payloads; test
  vectors from eel-crypto fixtures.
- Backoff: full jitter (master §2.5).

## Backend touchpoints

- Author `packages/openapi-contracts/field/poll-clerk.yaml` (9 paths from
  module spec) + bind in mobile-bff (voter/verification, station/status,
  incidents, config, sessions, push-tokens) + new BVV kiosk routes in bvi-svc.
- Events consumed: none; emitted: `clerk_checked_in`, `clerk_incident`,
  `end_of_day_report`, `voter_lookup_audit` (module table).

## UX / UI / ergonomics (screen-by-screen)

- **Shell**: `EelScaffold` with persistent bottom bar: Queue · Dashboard ·
  Incident · More; sync indicator chip top-right; station code + role in
  app bar; oversized text mode honored.
- **Login**: station code + clerk ID + PIN in one grouped card; biometric
  button prominent; attestation failure = full-screen blocked state with
  support code + phone number (FR copy from errors.json); no spinner-only
  states (each step labeled).
- **Dashboard**: 2-column stat grid (turnout, authed, battery, connectivity),
  large numbers (tokens typography.display), 30 s "updated Xs ago" caption;
  degraded = amber banner, offline = cached + "as of" timestamp.
- **Lookup**: 3-step flow (ID → justification → attestation) with progress
  rail; result card shows station + status only (no PII beyond necessary);
  rate-limit remaining shown as subtle chip; every audit field visible to DPO
  before submit ("This lookup will be logged: reason, time, your attestation").
- **Queue**: single list, priority section pinned top with distinct icon +
  label; call-next = primary button bottom-right (thumb arc); ticket issuance
  = large-print dialog; metrics row collapsible.
- **Incident**: category picker as icon grid (48 dp), photo/video chips with
  thumbnails, escalation selector locked after submit; submit = sticky bottom
  button with queue fallback ("Will send when online").
- **Kiosk**: high-contrast citizen-facing mode; staff controls in app bar
  overflow; consent screens use plain language + recorded checkbox.
- **EOD**: auto-compiled preview with per-section edit provenance ("from local
  records"); dual-signature sheet explains both signatures; after submit,
  receipt screen with id + timestamp.
- Empty/error/offline states per master §5.3; all copy from i18n bundle.

## Testing

- Unit: priority engine ordering (AC-M31-09 scenario), rate limiter, EOD
  aggregation (FR-023 fields), search benchmark (AC-M31-05), backoff, ETA.
- Widget: state matrix goldens for every screen; text-scale 2.0; hit targets.
- Integration: check-in→lookup→queue→incident→EOD happy path; offline incident
  + reconnect dedup (AC-M31-08, EC-DAT-01/02); background-kill at each FSM
  state (EC-MOB-01); JWT mid-expiry silent refresh (AC-M31-11, EC-IDN-02).
- Contract: Pact for all 9 poll-clerk paths + BFF paths used.
- Security: mitmproxy captive (AC-M31-07), rooted AVD block (AC-M31-15,
  AC-M31-06), forensic extraction shows no plaintext (AC-M31-14).
- Load: k6 92 k check-ins/60 min vs BFF (AC-M31-10).
- A11y: TalkBack/VoiceOver scripts for queue + incident (AC-M31-12).

## Definition of done

Master §9 checklist with app specifics: AC-M31-01…15 rows ticked with test IDs;
`poll-clerk.yaml` bound; allowlist entries for any remaining placeholders
removed; README rewritten to match reality (remove phantom flavors until
flavors actually exist); ODPC kiosk-review note tracked as open question with
owner.