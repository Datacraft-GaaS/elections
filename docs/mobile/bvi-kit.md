# BVI Kit (M05) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec: `docs/plans/modules/bvi-device.md`.
Est: 10 ED. Hardware: FAP60, AntIris, GNSS/IMU, printer.

## Current state

`services/biometric-edge-svc/apps/bvi-kit/` — 7 lib files. Real: time-fence
FSM logic, GNSS cross-check logic, mode switch, special-voter mode. Simulated
(stubbed): NTS time acquisition, GNSS readings, biometric capture loop
(`identification_workflow.dart`), no register store, no match engine wiring.

## Target features

| # | Feature | FR (M05) |
|---|---|---|
| F1 | Session open/close FSM (biometric + HSM countersign; window enforcement; s.55 emergency dual-control close; dual-control reset) | 001–005 |
| F2 | Voter identification: fingerprint 1:N against on-device register extract; fallback chain fp > iris > face > MRZ > name+ID | 006–007 |
| F3 | Match result UX: registered station display, name+photo confirm; REDIRECT with map/distance | 008–009 |
| F4 | Anti-double-voting session marks + refusal | 010 |
| F5 | Rejected-voter printable receipt (reason code/referral) | 011 |
| F6 | MRZ fallback after N biometric failures (ICAO validation + justification log) | 012 |
| F7 | Privacy: no templates outbound; only voter_id + modality + outcome | 013 |
| F8 | Quality gates: fingerprint ISO score, facial liveness CNN gate, worn-finger adaptive retry, amputee auto-path | 014–017 |
| F9 | NTS real time acquisition + time-fence FSM (exists — wire real source) | (time fence) |
| F10 | GNSS real readings (platform channel) + spoof guard (exists) | (EC-KIT-01) |
| F11 | On-device register: signed register extract bundle (BVR freeze snapshot) loaded via secure bundle; nightly delta | new |
| F12 | Participation token flow (per authenticated voter, batched to M10/M14) | (M08-012 shared) |
| F13 | Distress + kit reset flows | (M01 analog) |

## Files

```
lib/main.dart                        bootstrap (kit flavor)
lib/src/core/*                       kit-core: config, secure store, telemetry, outbox, hlc (kit-sdk)
lib/src/session/session_fsm.dart     EXTEND time_fence: real NTS client via platform channel (chrony/ntp fallback signed)
lib/src/register/register_store.dart NEW: Isar store of register extract (hash-chained pages)
lib/src/register/register_loader.dart NEW: bundle verify (Ed25519, BVR freeze manifest) + load + delta
lib/src/match/match_engine.dart      NEW: 1:N match via FAP60 SDK identify + threshold config
lib/src/identification_workflow.dart REWRITE: real capture + match + fallback chain FSM
lib/src/capture/{fingerprint,iris,face,mrz}_step.dart   real transports (kit-sdk/bvr-kit pattern)
lib/src/liveness/liveness_gate.dart  NEW: ONNX liveness via kit-l1 pattern (model bundle gated)
lib/src/receipt/receipt_printer.dart NEW: ESC/POS platform channel + layout
lib/src/tokens/participation.dart    NEW: bounded random token + batch signer
lib/src/security/distress.dart       duress binding
test/..., integration_test/..., golden/...
```

## Technical implementation

- **Register extract**: BVR freeze produces signed bundle (manifest: election,
  station, voter records hashed per page, Merkle root); kit verifies bundle
  signature + root against pinned BVR backend cert before import; store in
  Isar with page-level hash chain; delta updates are signed page diffs.
  1:N match: FAP60 `identify` (SDK handles gallery); threshold from policy
  (FMR target 1/10 000 default); on multi-hit → ambiguity screen.
- **Fallback chain FSM**: `fingerprint → iris → face → mrz → manual` each
  edge requires justification reason + logs; after N=3 consecutive biometric
  failures MRZ unlocks (FR-012); amputee flag short-circuits to face/ID
  (FR-017, `biometric_exception` set).
- **Liveness gate**: ONNX model bundle (blocked on WP-3.7 contract — same
  open item in status.md item 17); interface ready, CI runs with
  `SimulatedLiveness` that fails closed with explicit `liveness_unavailable`
  reason; device build loads real model.
- **NTS**: platform channel to system NTS/chrony (Android rooted kit image);
  fallback NTP with signed response check; failure → time-fence holds
  session (existing FSM refuses).
- **Receipt**: ESC/POS over USB/BT platform channel; layout template
  versioned; failure → reprint queue.
- **Participation tokens**: bounded random token (server-issued batch per
  station, signed; kit dispenses sequentially); count reconciliation vs
  auth count at close (FR-M08-013 analog); discrepancy → ECC alert.

## Algorithms

- Time-fence FSM (exists) + signed-time monotonic checks (EC-MOB-08).
- GNSS multi-constellation delta + IMU envelope (exists) with real inputs.
- 1:N thresholding + ambiguity handling; retry adaptivity for worn fingers
  (score decay per attempt, threshold relaxation bounded, s.44A handoff).
- Register page hash chain + Merkle root validation (kit-sdk verifier).

## Backend touchpoints

- bvr-svc freeze snapshot endpoints (register bundles); bvi-svc
  identifications/turnout routes exist (events consumed by BFF); BFF proxies
  for token batches + receipt audit.

## UX / UI / ergonomics

- **Idle → identify loop**: giant "Place finger" affordance; multi-voter
  queue-aware (kit is single-user authoritative — poll-clerk companion
  handles queue); result screens: PASS (station + name + photo + confirm
  timer), REDIRECT (map, distance, print option), REJECT (reason code +
  receipt), DUPLICATE (already authenticated at HH:MM).
- Voice guidance + haptics; 2-second re-auth guard; inactivity lock.
- Kiosk readability: huge type, high contrast, works with gloves.
- Operator mode screens (open/close/reset) are dual-control with two
  fingerprint prompts and explicit justification textareas.

## Testing

- Unit: fallback FSM edges, threshold adaptivity, token reconciliation,
  page-hash validation, time-fence boundaries.
- Widget: identify loop incl. every failure mode; receipt preview.
- Integration: register import → 1 000 synthetic identifies (synthetic
  driver) → session close; kill-drill; 48 h offline (register fully local).
- Device-gated: FAP60/AntIris/camera/printer real paths (self-check).
- Load: kit-simulator drives identification pattern (existing tool).
- Contract: Pact for token batch + event emission paths.

## Definition of done

Master §9; FR-M05-001…017 (plus kit FRs from bvi-device checklist) tested or
device-gated with prerequisites; no simulated-data paths in release flavor
(liveness model gate documented with tracker link); README updated.