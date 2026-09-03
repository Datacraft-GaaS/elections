# Accred Scanner (M42/M43) — Completion Plan

Master plan: `docs/mobile/plan.md`. Module specs: `media-accreditation-portal.md`,
`observer-accreditation-portal.md`. Backend: training-api accreditation
routes exist; **CRL endpoint does not** (app fetches `/v1/crl` against
`https://accred.eel.iebc.go.ke` — no such route). Est: 2 ED.

## Current state

`services/training-svc/apps/accred-scanner/` — 8 lib files. Real: QR badge
scan (mobile_scanner), CRL cache/staleness logic (5 min refresh, 30 min
stale), scan-result screens. Gaps: CRL backend route, pinning, auth, badge
verification against real `verify_badge` path, locales.

## Target features (completion)

| # | Feature | FR |
|---|---|---|
| F1 | Backend CRL endpoint (signed badge revocation list bundle) | new |
| F2 | Badge verify via `/api/v1/training/accreditation/badges/{badge_number}` (exists) with CRL cross-check | correctness |
| F3 | Offline verification: cached CRL + cached badge registry snapshot | spec |
| F4 | Pinning + session (scanner operator role) | standard |
| F5 | Scan UX hardening (dup-scan guard, low-light, manual entry) | UX |
| F6 | Locales | parity |

## Files

```
lib/features/crl/crl_repository.dart    point at /api/v1/training/accreditation/crl; verify bundle signature
lib/features/scan/scan_screen.dart      add manual entry, dup-guard, offline result states
lib/features/badges/badge_repository.dart NEW: verify_badge client + cache
lib/src/core/*                          api client + connectivity
l10n/                                   full set
```

## Technical implementation

- CRL bundle: `{generated_at, revoked:[badge_numbers], sequence, signature}`
  Ed25519-signed by training service; app verifies before accepting; stale
  threshold kept (30 min) but verification allows "stale CRL" decision mode
  with explicit warning + supervisor code (policy).
- Scan flow: QR → extract serial → local CRL check (revoked? stale?) →
  online verify_badge → result (valid/revoked/unknown/stale).

## Algorithms

- CRL sequence monotonicity guard (reject older sequence than cached).

## Backend touchpoints

- Add `GET /api/v1/training/accreditation/crl` to training-api + contract
  file; bind; Pact.

## UX / UI / ergonomics

- Full-screen scanner with frame; result card colour+icon+text (never
  colour-only): VALID (green ring), REVOKED (red, reason), STALE-CRL (amber
  warning with action), OFFLINE-UNVERIFIABLE (grey with queue option);
  manual serial entry for damaged badges; haptic per result; scan history
  with search.

## Testing

- Unit: CRL signature verify, sequence guard, staleness, result FSM.
- Widget: result states; RTL; locales.
- Integration: scan → verify (mock transport) incl. offline + stale paths.
- Contract: Pact for crl + verify_badge.

## Definition of done

Master §9; CRL path bound; no hard-coded external host without config;
README updated.