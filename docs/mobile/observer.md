# Observer App (M34) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/mobile-observer.md` (32 FRs). Contract to author:
`packages/openapi-contracts/field/observer.yaml` (11 paths per module table).
Est: 10 ED.

## Current state

`services/field-mobile-svc/apps/observer/` — 27 lib files; all 4 screens
(incident, station visit, mission export, tally verify) are literal stubs;
attestation + local db stubs; auth fake; 23 locales present.

## Target features

| # | Feature | FR |
|---|---|---|
| F1 | Enrolment (accreditation ID + biometric + attestation; category immutable) | 001 |
| F2 | MFA: FIDO2 or platform biometric + attestation; PIN fallback 5-fail lockout | 002 |
| F3 | Category-driven feature gating via SpiceDB (countersign rights UI) | 003 |
| F4 | Portal read: national/county/constituency/station results + signed 34A images; RTS with CDN fallback | 004 |
| F5 | Per-station Merkle proof retrieval + client verify via Rust FFI ≤ 60 s | 005 |
| F6 | Root-hash pinning vs transparency log; mismatch → automatic TAMPERING incident | 006, 026 |
| F7 | Station bookmarks/watch list + publish/update push | 007 |
| F8 | Bulk constituency download (rate-limited per category/hour, progress UI) | 008 |
| F9 | GPS check-in at station (polygon validation) + signed visit log | 009–011 |
| F10 | Time-boxed session + biometric sign-off on leaving | 010 |
| F11 | Structured incidents (7 categories) + evidence + GPS; encrypted queue; AG mirror on backend | 012–013 |
| F12 | Sampling workflow: n stations/constituency via cryptographic seed; protocol version recorded | 016 |
| F13 | Verification checklist per sampled station (4 Boolean items + notes) | 017 |
| F14 | Countersignature (category-permitted) anchored to P05 | 018 |
| F15 | End-of-mission report: auto-compiled + server-rendered PDF download; hash stored | 019–020 |
| F16 | Offline cache of snapshots/proofs with stale-since display | 022 |
| F17 | Systemic-incident flag (OBSERVER_CHIEF) exactly-once; SMS/USSD fallback | 015, 024 |
| F18 | Security envelope: pinning, FLAG_SECURE on incident screens, secure store, biometric per submission | 025, 027 |
| F19 | Locales: EN/SW launch + FR/PT pack on demand (signed) | 028 |
| F20 | WAL-journaled offline integrity; kill-resume | 030 |

## Files

```
lib/main.dart                      rewrite bootstrap
lib/src/core/*                     eel-core composition
lib/src/features/auth/             login (FIDO2/biometric), session
lib/src/features/portal/           results_browser (tabs: national/county/constituency/station), form34a_viewer
lib/src/features/verify/           station_verify_screen, proof_controller, root_pin_check, checklist_screen
lib/src/features/sampling/         sampling_controller (seeded selection), sampling_overview
lib/src/features/visit/            station_visit_screen (check-in/sign-off), visit_repository
lib/src/features/incident/         incident_screen (7 cats), systemic_flag_sheet (chief)
lib/src/features/bookmarks/        watchlist_screen, bookmark_repository
lib/src/features/bulk/             bulk_download_screen (queue + progress)
lib/src/features/mission/          mission_report_screen, mission_pdf.dart
lib/src/features/countersign/      countersign_sheet (gated)
lib/src/services/*                 DELETE stubs
test/..., integration_test/..., golden/...
```

## Technical implementation

- **F5/F6 verification**: kit-sdk `EelVerifierFfi` (libeel-verifier) verifies
  station form proofs and snapshot roots. Root hash pinned in bundle
  (`assets/config/root.json` at release; rotation = app update). Mismatch:
  auto-create TAMPERING incident draft pre-filled with root/station/observed
  hashes; station badge = `Unverifiable`, sticky until a fresh successful
  verification (never silently cleared).
- **F12 sampling**: seeded shuffle — `seed = SHA-256(protocol_version ‖
  constituency ‖ election_id)`; Fisher–Yates over station list with that
  RNG; take n from policy bundle; protocol version recorded in every record.
  Deterministic across reinstall (pure function; golden test).
- **F9 polygon check**: point-in-polygon (ray casting) against station
  polygon from BFF config; tolerance ring; distance displayed.
- **F8 bulk**: per-category hourly bucket from policy; sequential chunk
  downloads (forms + proofs) into Isar blob store; resumable ranges;
  progress list with per-item retry; storage-full guard (EC-MOB-02).
- **F15 PDF**: server-side render (`GET .../end-of-mission/pdf`); client
  caches + stores SHA-256 in the mission record; open/share sheet.
- **F17 SMS fallback**: systemic flag composes a deep-linkless minimal
  payload via platform SMS intent to reserved short code; IVR ack confirmation
  status tracked in-app (best effort; documented).

## Backend touchpoints

- `observer.yaml` (11 paths) bound in mobile-bff; rts/portal pass-throughs;
  AG mirror adapter on backend (M34-032); P05 anchoring on countersign.

## UX / UI / ergonomics

- **Tabs**: Results · Stations · Incidents · Mission.
- **Results browser**: hierarchical drill (national → county → constituency →
  station); each row shows published badge + proof status ring; stale-since
  captions; 34A image viewer with pinch zoom + hash footer.
- **Verify screen**: 4-item checklist as large toggle rows with notes
  affordance; verdict summary ("Form matches · agents signed · polygon ok ·
  seals ok"); Verify button runs proof; Countersign appears only for
  permitted categories (SpiceDB flag surfaced in profile).
- **Station visit**: check-in card with polygon map pin (offline tiles),
  session timer, sign-off with biometric; notes field autosaves.
- **Sampling**: constituency card shows "n=12 sampled" with regenerate
  disabled (seed-locked) + protocol version chip.
- **Incidents**: same pattern as poll-clerk but 7 observer categories;
  systemic flag = destructive-style confirm + SMS fallback suggestion when
  offline.
- **Mission**: stats grid + "Generate PDF" (server) + download list of past
  reports with hashes.
- Full state matrix; goldens; dark mode; TalkBack traversal of verify +
  incident flows (AC list in module spec).

## Testing

- Unit: seeded sampling determinism + distribution, polygon containment,
  checklist aggregation, stale-since logic, bulk bucket limiter.
- Widget: all screens × states; Unverifiable badge sticky behaviour.
- Integration: check-in → checklist → verify → countersign ≤ 60 s (FR-025);
  offline cached verification; mismatch → auto-incident; kill-drill; bulk
  resumable download over flapping netem.
- Contract: Pact for 11 paths. A11y: traversal + pseudo locale + FR/PT pack
  load. Security: pinning captive test, FLAG_SECURE, biometric per submit.

## Definition of done

Master §9; FRs 001–032 tested; AC rows ticked; FR/PT pack signed-delivery
path tested in staging; contract bound; MDM staging install verified.