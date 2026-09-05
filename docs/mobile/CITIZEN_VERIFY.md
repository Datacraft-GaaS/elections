# Citizen Verify App — Complete Reference (M13/M29)

**Code:** `services/field-mobile-svc/apps/citizen-verify/`
**Module spec:** M13 (public verifier) + M29
**Plan:** [`docs/mobile/plan.md`](plan.md)
**Contract:** `packages/openapi-contracts/field/citizen-verify.yaml`
**Tier:** T1 public (10 M concurrent), soft attestation (never blocks)

---

## Purpose

Public-facing verification app for citizens. Read-heavy: own register lookup, polling station finder, per-station results with signed Form 34A + client Merkle verification, incident reporting, bookmarks. No forced MDM or hard attestation.

---

## Functions (10 Features)

### C1 — Onboarding + Consent (M29)
Data notice, no-tracking attestation (`attestation: soft-warn` tier, never blocks). Consent recorded locally.

### C2 — OTP Login (M13)
Phone number → SMS OTP via BFF (`POST /api/v1/mobile/bff/otp/request|verify`). Rate: 20/day/session. Token = 15-min session JWT scoped to own-record reads.

### C3 — Register Lookup (M13 FR-010–012)
Own record: status, station assignment, queue hints. Justification + DPO biometric attestation required. Rate-limited (token-bucket, station/hour from policy). Permanent audit record.

**Objection Flow** (transfer/removal/rectification): BVR integration via `POST .../objection` with justification + biometric attestation.

### C4 — Station Finder (M13)
GPS + search (offline map pin from cache). Results show station code, address, distance from current location.

### C5 — Results Browser (M13/M34 shared verifier)
National → county → constituency → station. Each row shows status + proof ring. Client-side Merkle proof verify (`EelVerifierFfi`) with visible result: **VERIFIED** (green ring) / **NOT FOUND** / **INVALID** (red ring) + proof details sheet.

### C6 — Snapshot Root Check (M29)
Pinned root (`assets/config/root.json`) vs server. Mismatch → automatic TAMPERING incident (auto-filled, anonymous session).

### C7 — Citizen Incident Report (M29)
Anonymous (`reporter_session_id`: random UUID, weekly rotation). Optional photo. GPS only with explicit consent toggle. Encrypted queue to BFF.

### C8 — Bookmarks (M29)
Local-only, hashed session. No PII transmitted. Bookmarks include station code, timestamp, user note.

### C9 — Audio Guide (M13)
Results verification flow guided by EN/SW audio (`assets/audio/`). Transcript sync verified in CI.

### C10 — USSD/PWA Parity (Architecture)
Same BFF tier-1 paths serve USSD and PWA clients. App verifies parity against PWA contract fixtures.

---

## Technical Implementation

- **Verification UX**: Reuse `kit-sdk` `EelVerifierFfi`. If FFI unavailable (older devices), fall back to pure-Dart verifier with visible "software verifier" chip. Both must agree on all CI fixtures.
- **OTP**: BFF paths rate-limited; token session scoped to `own_record_read` only.
- **CDN-first**: Results read hits tier-1 endpoints (`/v1/public/results/*`) with `Cache-Control: public, max-age=300`. App verifies `ETag` + hash before display.
- **Aggressive caching**: Station results + 34A images + proofs cached in Isar (LRU 200 MB cap, content-addressed by SHA-256). Stale-since annotations (`EelStatusChip`) everywhere.
- **Offline**: Cache-first with fallback to CDN; proof verification works against cached snapshot; mismatch reported when reconnect.

---

## Design System Reference

Every screen uses: `EelScaffold` (scrollable variant), `EelButton` (primary with icon), `EelResultCard` (station/info card), `EelStatusChip` (verified/not-found/invalid/stale), `EelTextField` (search/OTP input), `EelBanner` (offline warning, data-saver info), `EelSealStamp` (verification trust indicator), `EelTextStyle` (large, accessible typography), `EelColors.of(context)`.

---

## Testing

- **Unit**: LRU cache eviction, rate limiter token bucket (20/day), verifier parity fixtures (must match FFI output), anonymous session rotation.
- **Widget**: All screens × states (cached/offline/fresh); 2G image-defer mode; text scale 200 %.
- **Integration**: OTP → lookup; results browse offline with cached proofs; incident consent path; bookmark sync.
- **Contract**: Pact tier-1 paths + OTP endpoints.
- **A11y**: Full TalkBack sweep + audio-guide transcript sync.

---

## Security / Privacy

- No forced MDM; no device fingerprinting beyond `soft-warn` attestation.
- Anonymous incident: no PII linked; photo optional; GPS only with consent.
- Bookmarks: hashed session key only (not user identity); local-only storage.
- No advertising or tracking SDKs; data notice explains BFF tier-1 usage only.
