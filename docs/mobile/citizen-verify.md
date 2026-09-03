# Citizen Verify (M13/M29) — New App Plan

Master plan: `docs/mobile/plan.md`. Module specs:
`docs/plans/modules/public-verifier-app.md` (M13) +
`mobile-citizen-verify.md` (M29). **The app does not exist** though both
specs declare `services/field-mobile-svc/apps/citizen-verify/`. Tier T1,
public-facing (10 M concurrent), soft attestation only (no MDM). Est: 8 ED.

## Scope

Read-heavy PWA-grade Flutter app: own register lookup (OTP-verified),
polling-station finder, per-station results with signed Form 34A + client
Merkle verification (Rust FFI), objection filing, citizen incident reports,
bookmarks. Languages: EN/SW launch (+ phased Kenyan locales); audio guide
EN+SW for results flow.

## Architecture deltas vs persona apps

- **No forced MDM/attestation**: bootstrap skips hard attestation
  (`attestation: soft-warn` tier recorded, never blocks); anonymous session
  with rate-limit token from BFF.
- **OTP auth**: phone number → SMS OTP via BFF (public path) for register
  lookup + objections; results/proof paths are public tier-1 (cached CDN).
- **Aggressive caching**: station results + 34A images + proofs cached in
  Isar (LRU 200 MB cap, content-addressed); stale-since everywhere.

## Features

| # | Feature | Source FRs |
|---|---|---|
| C1 | Onboarding + consent (data notice, no-tracking attestation) | M29 |
| C2 | OTP login (own-record lookup rate 20/day/session) | M13 API |
| C3 | Register lookup card (status, station, queue hints) + objection flow (rectification/transfer/removal via BVR) | M13 |
| C4 | Polling station finder (GPS + search, offline map pin) | M13 |
| C5 | Results browser: national → county → constituency → station; 34A image viewer; **client-side Merkle proof verify** (FFI) with green/red result + proof details sheet | M13/M34 shared verifier |
| C6 | Snapshot root check vs pinned roots + mismatch reporting | M29 |
| C7 | Citizen incident report (anonymous session id, photo, consent-gated GPS) | M29 |
| C8 | Bookmarks (local only, hashed session) | M29 |
| C9 | Audio guide for verify flow (EN/SW, a11y) | M13 |
| C10 | USSD/PWA parity note: same data via BFF tier-1 paths | arch |

## Files

```
services/field-mobile-svc/apps/citizen-verify/   NEW app
lib/main.dart
lib/src/core/*            (eel-core, anon session variant)
lib/src/features/onboarding/
lib/src/features/otp/         otp_screen, otp_repository
lib/src/features/register/    lookup_screen, objection_flow
lib/src/features/stations/    finder_screen
lib/src/features/results/     results_browser, form34a_viewer, proof_sheet
lib/src/features/incident/    report_screen (anon)
lib/src/features/bookmarks/
l10n/ (en,sw + 9 phased)  assets/audio/{en,sw}/verify_*.mp3
test/..., integration_test/..., golden/...
```

## Technical implementation

- Verification UX: reuse kit-sdk `EelVerifierFfi`; when FFI unavailable
  (older devices) fall back to pure-Dart verifier with visible
  "software verifier" chip; both must agree on fixtures (CI property test).
- OTP: BFF `POST /api/v1/mobile/bff/otp/request|verify` (new paths, rate
  limited); token = 15-min session JWT scoped to own-record reads.
- CDN-first: results read path hits BFF tier-1 endpoints designed for CDN
  (cache-control headers tested by contract).
- Incident: anonymous `reporter_session_id` (random UUID rotated weekly),
  no PII; photo optional; location only with explicit consent toggle.

## Backend touchpoints

- `citizen-verify.yaml` contract authored (M13 table: register, objection,
  station, results, proof, incidents) + BFF handlers (mostly pass-through to
  bvr/vprs/rts + transparency proofs); Karapace consumer tests.

## UX / UI / ergonomics

- Consumer-grade simplicity: one primary action per screen; giant Verify
  button; results with progress ("counted 87 % of stations"); proof result as
  simple VERIFIED/NOT FOUND/INVALID ring + "what is this?" link (audio
  explains); offline-friendly (cached results marked with date); data-saver
  toggle (skip images on 2G, EC-CON-03).
- WCAG 2.2 AA strict (public app): text scaling 200 %, screen-reader labels
  on every proof element, contrast from tokens; low-literacy copy (plain
  language, tested per content guidelines); phone-number entry with keypad
  optimization.

## Testing

- Unit: LRU cache, rate limiter display, verifier parity fixtures.
- Widget: all screens × states; 2G image-defer mode.
- Integration: OTP → lookup; results browse offline with cached proofs;
  incident consent path.
- Contract: Pact tier-1 paths + OTP; k6 10M-read profile against BFF+CDN
  (backend AC, app contract-tested).
- A11y: full sweep + audio-guide transcript sync test.

## Definition of done

Master §9; contract bound; app listed in service README surfaces table;
MDM not applicable — public store tracks (internal first) + PWA parity
checked.