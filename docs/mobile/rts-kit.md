# RTS Kit (M08) — Completion Plan

Master plan: `docs/mobile/plan.md`. Module spec: `docs/plans/modules/rts-device.md`.
Backend `rts-device-api` routes exist. Est: 4 ED.

## Current state

`services/rts-svc/apps/rts-kit/` — 12 lib files. Real: scan quality gates,
Ed25519 sign, mTLS HTTP transmit to `/api/v1/rts/forms` with retries,
crash-consistency queue, a11y semantics, mode switch, OCR preview (never
blocks capture). Stub: SMS fallback (`transmit.dart` — documented, logs and
pretends success).

## Target features (completion)

| # | Feature | FR | Status |
|---|---|---|---|
| F1 | Real SMS fallback: CBOR+zstd → 160-char segments via SIM toolkit/AT channel platform bridge; server reassembly endpoint exists (`form34a/sms-fallback`) | M08 SMS FRs | stub → real |
| F2 | All 6 position pipelines independent (34A/35A/36A/37A/38A/MCA) | 002, 006 | partial |
| F3 | QR embed at scan binding station/kit/form/event | 004 | check + test |
| F4 | PO biometric sign gate (NFIQ threshold) | 009 | partial |
| F5 | Party-agent countersign window (30 min default, addable, auto-close log) | 010, 014 | missing |
| F6 | Participation tokens + reconciliation | 012–013 | missing in app |
| F7 | Duress finger covert alert | 011 | kit-sdk hook — wire |
| F8 | Local encrypted retention until end-of-cycle | 008 | partial |
| F9 | Dual-SIM resubmit idempotency | backend exists | wire |

## Files

```
lib/src/rts_workflow/transmit.dart     REPLACE SMS stub with SmsTransport (platform bridge + segmenter)
lib/src/sms/sms_transport.dart         NEW: platform channel to SIM/AT bridge; segmenter + reassembly ack
lib/src/rts_workflow/pipelines.dart    NEW: per-position pipeline manager (6 lanes)
lib/src/rts_workflow/countersign.dart  NEW: window timer + countersign capture
lib/src/rts_workflow/tokens.dart       NEW: participation token batch + reconciliation
lib/src/rts_workflow/duress.dart       wire kit-sdk EelDuressHook
lib/src/rts_workflow/queue.dart        EXTEND: dual-sim resubmit idempotency mode
test/sms_segment_test.dart, pipelines_test.dart, countersign_window_test.dart, tokens_test.dart
```

## Technical implementation

- **SMS transport**: `SmsSegmenter`: payload → CBOR (SignedForm) → zstd →
  base32; chunks 152 chars + 8-char header (seq, total, batch crc16);
  send via platform channel `eel.rts.sms/send` (native: SIM toolkit or
  `SmsManager` on kit build); ack = server receipt id per batch
  (`transmission-attempt` ledger already exists); un-acked segments → resend
  queue. Budget: ≤ 5 SMS segments / form / 10-min window enforced client-side
  (queue if exceeded).
- **Pipelines**: `PositionPipeline` per elective position: independent state
  FSM (capture → quality → sign → transmit → retained), failure isolation
  (one lane's error never blocks others — FR-006), shared transmitter with
  per-lane queues; parallel bounded to 2 concurrent uploads on 3G.
- **Countersign window**: timer from PO sign (default 30 min, config);
  agent biometric/PIN capture adds countersignature to bundle + retransmit
  patch; expiry → logged event + UI lock.
- **Tokens**: batch fetch per station (endpoint exists), dispense one per
  authenticated voter, count reconcile at close (FR-012–013); discrepancy →
  ECC alert event.
- **Retention**: signed image encrypted in Isar; purge job at
  end-of-election marker from config.

## Algorithms

- zstd via `es_compression`/FFI or pre-zstd CBOR+deflate fallback gated by
  server capability flag (contract already allows CBOR+zstd).
- CRC16/CCITT per batch; segment reassembly table server-side (exists).
- Reconciliation: exact count match; off-by-N report includes HLC window.

## Backend touchpoints

- rts-device-api: sms-fallback returns segment ack receipts (extend
  response), dual-sim-resubmit used on SIM failover (exists); BFF not in the
  hot path (device API direct with pinning).

## UX / UI / ergonomics

- Position tabs with per-lane status rings; transmission health banner
  (HTTP ok / SMS fallback active + segments used); PO sign = biometric sheet
  with NFIQ feedback; scan guidance overlays; a11y semantics already
  scaffolded — extend per new screens; wakelock during transmit (existing).
- Offline-by-design: forms retained + retried; nothing blocks capture
  (FR: capture never blocked on inference/network).

## Testing

- Unit: segmenter round-trip (random payloads, sizes), budget limiter,
  pipeline isolation, window expiry, reconciliation math.
- Widget: lanes UI, SMS banner states, countersign capture.
- Integration: transmit over toxiproxy flap → SMS fallback triggers at 3
  failures (FR threshold), ack + dedup; kill-drill mid-sign.
- Contract: Pact device-api paths (upload, sms-fallback, transmission-attempt,
  tokens, dual-sim-resubmit).
- A11y: existing workflow semantics extended; 11 locales complete.

## Definition of done

Master §9; SMS stub removed (transmit.dart header docs updated to describe
real bridge); FR-M08-001…014 tested or device-gated; README updated.