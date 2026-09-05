# BVR Kit (M01) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec: `docs/plans/modules/bvr-device.md`.
Est: 8 ED. Hardware: FAP60 fingerprint reader (integrated), passport MRZ
camera, (optional) AntIris iris.

## Current state

`services/biometric-edge-svc/apps/bvr-kit/` — 5 lib files. Real: FAP60
fingerprint driver (Fap60SensorDriver → kit-sdk FingerprintScanner →
`FingerprintPlugin.kt`), i18n, mock fallback. Stub: MRZ scan, IPRS verify,
sign, upload are `Future.delayed` chains (`enrolment_workflow.dart:103-109`);
no local store; no offline queue; no ABIS ingest.

## Target features

| # | Feature | FR (M01) |
|---|---|---|
| F1 | 22-field biographic capture with client validation | 001, 008 |
| F2 | 10-print fingerprint capture (FAP60) with ISO 19794-2 minutiae + WSQ/JPEG2000 encode | 002 |
| F3 | Iris capture (AntIris via kit-sdk IrisScanner + IrisPlugin.kt) ISO 19794-6 + liveness gate | 003 |
| F4 | Face capture ICAO 9303 + quality gate (pose/lighting/expression) | 004 |
| F5 | Signature pad (vector hash only) | 005 |
| F6 | Document scan: 2nd-gen ID / Maisha / passport; MRZ parse + check digits; encrypted PDF to MinIO via BFF | 006 |
| F7 | On-device ABIS-lite dedup (last-N station cache, LRU) + server dedup flag | 007 |
| F8 | Offline-first queue (7-day ring, signed records, gap audit) | 009–013 |
| F9 | Session: fingerprint login (PO roles), PIN fallback, inactivity lock, role per SVID | 014–016 |
| F10 | Distress finger + honeypot revoked-kit mode | 017–018 |
| F11 | s.44A dual-officer assisted path + amputee path | (from app flow) |

## Files

```
lib/main.dart                       bootstrap via eel-core (kit flavor)
lib/src/core/*                      local_db (Isar enrolment queue), outbox, api_client (BFF/ABIS), config
lib/src/enrolment_workflow.dart     REWRITE: real step FSM
lib/src/steps/biographics_screen.dart   22-field form + validation
lib/src/capture/fingerprint_step.dart   wire real driver + quality gate + ISO encode (kit-sdk)
lib/src/capture/iris_step.dart          AntIris via kit-sdk IrisScanner
lib/src/capture/face_step.dart          camera + quality gate (see algorithms)
lib/src/capture/mrz_step.dart           mobile_scanner + MRZ parser (ICAO 9303 check digits)
lib/src/capture/document_pdf.dart       encrypted PDF packaging + MinIO upload via BFF
lib/src/capture/signature_pad.dart      vector capture + hash discard
lib/src/dedup/abis_lite.dart            on-device dedup (see algorithm)
lib/src/queue/enrolment_queue.dart      signed WAL queue (kit-sdk offline_queue + Isar)
lib/src/session/            fingerprint login + lock
lib/src/security/distress.dart          kit-sdk duress hook binding
android/... (exists)                    extend IrisPlugin for capture session
test/..., integration_test/..., golden/...
```

## Technical implementation

- **Workflow FSM** replaces the 4 delays: states `biographics →
  fingerprint(×10) → iris → face → documents(mrz) → signature → dedup →
  sign → queue`. Each step: journal intent (Isar WAL) → capture → quality
  gate (fail → guided retry with reason) → next. `s.44A` dual-officer path:
  when biometrics fall below threshold after 3 retries, screen offers
  assisted path requiring both officers' fingerprints + reason.
- **ISO encoders**: kit-sdk gains `Iso19794_2` (minutiae template build from
  FAP60 SDK output), `Iso19794_6` (iris), `Iso19794_5` (face) helpers —
  native libs output raw; Dart wrapper validates lengths/checksums; WSQ +
  JPEG2000 encode via FAP60 SDK / `flutter_image_compress` fallback + server
  re-encode guard.
- **MRZ**: implement ICAO 9303 TD1/TD2/TD3 parser with check-digit validation
  (7-3-1 weighting); capture via `mobile_scanner`; crop + enhancement before
  parse; expiry ≥ 6 months check (FR-008).
- **ABIS-lite dedup**: feature vector = 64-dim projection of fingerprint
  template (fixed downsample of minutiae histogram — documented approximation);
  cosine similarity against station cache (LRU, N=500 enrolments); score ≥
  0.93 → `possible_duplicate` flag (never blocks; server ABIS decides);
  cache-miss always flags server-side check.
- **Queue**: per master §2.5; enrolment record = biographic hash + biometric
  blob refs + kit serial + clerk biometric id + HLC; Ed25519 signature over
  canonical JSON; 7-day capacity check (EC-MOB-02 analog); gap audit on
  reconnect (signed sequence).
- **Honeypot**: if kit SVID revoked flag from BFF → sandbox UI + silent
  biometric capture of attacker (outboxed as `honeypot_capture`).

## Algorithms

- ICAO 9303 MRZ check digits (7-3-1) — unit-test with sample passports.
- Minutiae-histogram projection dedup (above) with fixed seed; recall on
  synthetic corpus ≥ 0.85 measured in CI fixture (server ABIS remains
  authority).
- Face quality gate: brightness/blur/sharpness heuristics (Laplacian
  variance + histogram) client-side; final quality at ABIS.
- Fingerprint quality: NFIQ2-equivalent score from SDK; threshold 3/5.
- Canonical JSON + Ed25519 (kit-sdk); HLC timestamps.

## Backend touchpoints

- ABIS kit ingest exists (`abis_kit_client.dart` → `abis-api` kit routes) —
  wire enrolment records through BFF → ABIS with dedup job ids; BVR backend
  enrolment endpoints (`bvr-api` routes/enrolments.rs exist) for record
  persistence; document upload via presigned MinIO flow.

## UX / UI / ergonomics

- **Guided capture**: full-screen steps with oversized progress rail
  (1/7), large illustrations per modality, voice guidance button (kit-sdk
  `EelVoiceGuideButton`), quality feedback as plain language + icon ("Lift
  finger slightly — press lighter"), auto-advance on pass.
- **Biographics**: 22 fields grouped in 4 sections with section progress;
  dropdowns for county/constituency/ward (offline gazette data); validation
  errors inline per field (alpha-only names, DOB ≥ 18, expiry rules); NFC
  text normalization.
- **Document step**: camera frame with MRZ highlight; retake with reason.
- **Queue/sync**: persistent status bar (pending count, last sync, last
  error code — FR-013); low-storage warning with action.
- Offline is default posture; every screen operable without network; sync
  events visible but non-blocking. Glove/touch ergonomics: 56 dp targets in
  capture flows; high-glare theme (forced light, max contrast).

## Testing

- Unit: MRZ vectors, ISO encoder validation, dedup recall fixture, queue
  capacity + gap audit, signature canonicalization.
- Widget: step FSM incl. quality-fail retry loops, s.44A path, honeypot.
- Integration: full enrolment → offline → reconnect dedup; kill-drill each
  FSM state; 7-day fill simulation.
- Device-gated (self-check with `--check-hardware`): FAP60 real capture,
  AntIris real capture, real camera MRZ, PDF/MinIO upload. CI uses synthetic
  transports; device tests in `.woodpecker/mobile.yaml` device-nightly job.
- Contract: Pact for enrolment + ABIS kit ingest paths.

## Definition of done

Master §9; FR-M01-001…018 tested (device-gated rows listed with hardware
prereq + self-check); workflow FSM has zero `Future.delayed` stubs; README
updated; MDM kit profile install verified.Updating docs/mobile/bvr-kit.md -> BVR-KIT (expanded reference with functions, capabilities, design-system usage, testing)

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
