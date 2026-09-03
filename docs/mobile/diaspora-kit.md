# Diaspora Kit (M24) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/diaspora-voting.md` (+ physical logistics M65 overlaps
with custody-mobile plan). Est: 5 ED.

## Current state

`services/biometric-edge-svc/apps/diaspora-kit/` — 9 lib files. Real:
deferred-transmission queue concept, station/model screens, timezone
display. Fabricated: `hsmSignature: 'stub-kit-signature-...'`
(`form34a_screen.dart:56`) — must be replaced with real signing or removed.

## Target features

| # | Feature | FR (M24) |
|---|---|---|
| F1 | DIASPORA_BVR mode provisioning: embassy + mission country scope lock | 001 |
| F2 | 22 fields + diaspora extras (host-city, ID/passport, embassy reg date) | 002 |
| F3 | Biometric capture (10-print, iris, face) encrypted at rest; delayed batch (not live) | 003 |
| F4 | Embassy-local dedup in-session; server ABIS dedup on batch | 004 |
| F5 | Registration window enforcement (gazetted dates) | 005 |
| F6 | Voter roll freeze acknowledgement (14 days prior, signed snapshot) | 006 |
| F7 | Locales: EN/SW + Arabic/French on voter-facing screens | 007 |
| F8 | Form 34A capture + deferred transmission (existing queue, real signatures) | election-day FRs |
| F9 | Ballot pouch scan + packing hand-off to custody-mobile (M65 boundary) | M65 |

## Files

```
lib/main.dart                       bootstrap (kit flavor, diaspora profile)
lib/src/core/*                      kit-core: db/outbox/crypto/config
lib/src/scope/provisioning.dart     NEW: embassy scope lock (MDM managed config + server verify)
lib/src/forms/form34a_screen.dart   REWRITE signing: kit-sdk EelSigningKey Ed25519 (device key), signature chain shown
lib/src/queue/deferred_queue.dart   EXTEND: encrypted-at-rest entries, batch manifest, delayed-batch upload
lib/src/register/roll_freeze.dart   NEW: freeze snapshot display + acknowledgement signature
lib/src/capture/*                   import kit capture steps (bvr-kit pattern) for diaspora mode
lib/src/i18n/*                      add ar/fr locale packs (signed bundle)
test/...
```

## Technical implementation

- **Real signing**: replace stub string with kit-sdk `EelSigningKey.sign`
  (device-bound Ed25519; attestation tier recorded); signature covers form
  canonical JSON + station + HLC; the "HSM-backed" label in UI only when
  verdict tier ≥ strongBox else `software-backed` chip (honest tiering per
  kit-sdk attestation).
- **Deferred batch**: batches close at `deferredTransmissionAt`; manifest =
  list of record ids + HLC range + batch Ed25519 signature; upload
  `POST /api/v1/rts/device/diaspora-batch` (new rts-device-api endpoint §4.3
  of master); idempotent per batch id; embassy offline window (days) is
  normal — UI shows "batches sealed, transmit at embassy office".
- **Scope lock**: managed config supplies `{embassy_code, mission_country,
  window:[from,to], roll_frozen_at}`; app verifies server-side countersigned
  copy at session start; mismatch → blocked (tamper-resistant activation).
- **Roll freeze ack**: officer acknowledges frozen roll with biometric +
  signature; recorded event; kit refuses enrolment after freeze (window +
  freeze both enforced).

## Algorithms

- Batch manifest signing + idempotent batch upload (retry with same batch
  id returns cached receipt).
- Window/fence checks reuse kit-sdk TimeFence with diaspora policy values.

## Backend touchpoints

- rts-device-api diaspora-batch endpoint; BVR diaspora mode config service;
  ABIS delayed-batch ingest (exists as abis kit contract).

## UX / UI / ergonomics

- Embassy-officer-first: same patterns as bvr-kit capture screens;
  voter-facing tally entry screens in AR/FR/SW/EN with RTL layout for AR;
  batch status screen (sealed/pending/transmitted + manifest viewer);
  timezone-aware timestamps displayed in both local (embassy) and Nairobi
  time (existing tz_display extended).
- Clear dual timestamps + "offline by design" messaging; no connectivity
  indicators implying realtime sync.

## Testing

- Unit: batch manifest signing/verification, window enforcement edges
  (T-14 freeze), scope lock mismatch, RTL text fixtures.
- Widget: form34A with real signature states (software vs hardware tier
  chips), batch seal/transmit flow, freeze ack.
- Integration: enrolment → batch seal → (simulated) transmit → dedup;
  kill-drill; 7-day offline fill.
- Contract: Pact for diaspora-batch + enrolment paths.
- A11y: AR locale pseudo-sweep + RTL golden.

## Definition of done

Master §9; no fabricated signatures anywhere (`stub-kit-signature` removed;
scanner green); FRs tested; staging embassy-profile install verified.