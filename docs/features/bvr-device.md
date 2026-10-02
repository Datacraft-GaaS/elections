# BVR device (M01)

Module M01 (`bvr-device`) owned by `biometric-edge-svc`. Captures biographic and biometric data for every citizen registering to vote, operating offline-first on KIEMS kits and legacy Android devices. Guarantees zero record loss or tampering between capture and BVR back-end acceptance.

## Contents

1. What M01 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M01 does

M01 is the field-facing enrolment app running on ~65,000 new Android KIEMS kits and ~14,100 legacy Android devices. It captures 22 biographic fields, 10-print fingerprints, both irises, a facial image, a stylus signature, and an ID document scan; applies on-device quality gates; queues records locally for up to 7 days; and submits them to M02 (bvr-backend) on reconnect.

M01 is distinct from M02 (server-side register) and M03 (full 1:N dedup). M01 includes an on-device ABIS-lite matcher to catch obvious station-level duplicates before submission; definitive 1:N dedup runs at M03.

## 2. Data plane

Aggregates (per plan `docs/plans/modules/bvr-device.md` + `services/biometric-edge-svc/crates/edge-api/src`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **Voter** | voter_id (UUID7), 22 biographic fields, status, version | A |
| **BiometricTemplate** | template_id, voter_id, modality, ISO format, WSQ blob, field-encrypted | A |
| **IDDocumentScan** | scan_id, voter_id, MRZ parsed, PDF hash, encrypted PDF ref (MinIO) | A |
| **Signature** | sig_id, voter_id, stylus vector hash (raw vectors discarded post-hash) | A |
| **AuditEvent** | event_id, actor, action, ts_hlc, kit_serial, station_code | C |
| **KitAttestation** | kit_serial, firmware_digest, TPM quote, sent_at | C |
| **KitCustodyEvent** | kit_serial, from_officer, to_officer, seal UIDs, ts | C |

Kit-side store: encrypted SQLite WAL queue. Biometric templates field-encrypted via `packages/eel-crypto-rs` before write.

## 3. Ingest sources

| Source | Notes |
|---|---|
| 10-print flat-bed sensor (FBI PIV IQS / STQC certified, FAP 50/60) | ISO/IEC 19794-2 minutiae; WSQ at 500 PPI (FBI spec); JPEG-2000 (ISO 15444-1:2019) at 15:1 |
| Dual iris camera | ISO/IEC 19794-6 encoding; liveness before template extraction |
| Front camera (facial) | ICAO 9303 / ISO 19794-5 quality gate (pose, lighting, expression) |
| Stylus tablet | Vector hash only; raw vectors discarded |
| ID scanner (2nd-gen NID, Maisha card, passport) | MRZ parse; double-feed detection; encrypted PDF to MinIO |
| Operator (22 biographic fields) | Client-side NFC normalisation, check-digit validation, DOB >= 18 |

NFIQ 2.2 quality gate per NIST SP 500-333 (rejection threshold configurable); iris liveness via `packages/eel-biometric-rs`.

## 4. Pipeline

Crates involved: `services/biometric-edge-svc/crates/edge-api/`, `crates/kit-attest/`, Flutter apps under `services/biometric-edge-svc/apps/bvr-kit/`.

```
clerk biometric login (fingerprint / PIN fallback, rate-limited)
      -> station + window validation (geo-fence + gazetted time)
      -> biographic capture (22 fields, MRZ pre-populate)
      -> fingerprint capture x10 (NFIQ 2.2 gate, 3 retries)
      -> iris capture x2 (liveness gate)
      -> face capture (ICAO pose/illumination gate)
      -> signature capture (stylus vector hash)
      -> ID document scan (MRZ parse, encrypted PDF to MinIO)
      -> on-device ABIS-lite 1:N against station cache
             hit -> flag for server dedup, do not block
      -> sign record: Ed25519 binding voter_id + kit_serial
                      + clerk biometric ID + HLC timestamp
      -> append to encrypted SQLite WAL queue
      -> KitAttestation (TPM quote + firmware digest) -> edge-api
      -> on reconnect: batch sync to bvr-backend (M02)
             Idempotency-Key per record; zero duplicates
      -> emit eel.bvr.enrolment_captured.v1 (Kafka eel.bvr.voter.v1)
```

Offline queue ring-buffer covers 7 days of peak throughput. A/B OTA partitioned slots with automatic rollback on boot failure.

## 5. API surface

Contract: `packages/openapi-contracts/biometric-edge/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /bvr/v1/enrolments` (gRPC) | Kit (SVID mTLS) | Submit signed enrolment batch to bvr-backend |
| `POST /bvr/v1/attestations` (gRPC) | Kit (SVID mTLS) | Boot-time TPM quote + firmware digest |
| `POST /bvr/v1/transfers` (gRPC) | Kit (SVID mTLS) | Voter transfer events |
| `GET /bvr/v1/handoff/{token}` | Accredited 3rd party (SVID mTLS) | Fetch VPRS pre-filled form |
| `POST /edge/v1/ota/manifest` | MDM (SVID mTLS) | Push signed OTA manifest |
| `POST /edge/v1/distress` | Kit (SVID mTLS) | Internal distress signal relay |

All writes carry `Idempotency-Key`; server dedup via PostgreSQL-durable idempotency store.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| BVR kit app (primary) | `services/biometric-edge-svc/apps/bvr-kit/` (Flutter) | Enrolment workflow for ~65k new kits |
| Legacy Android shim | `services/biometric-edge-svc/apps/bvr-kit/` (Flutter + degradation layer) | Feature-matrix degradation on ~14,100 legacy devices |
| Kit attestation backend | `services/biometric-edge-svc/crates/edge-api/` (Rust/Axum) | Kit-facing REST + gRPC endpoint |

Languages: English + Kiswahili on all surfaces; architecture extensible to 11 languages (ICU plural rules, no code change). WCAG 2.1 AA throughout; TalkBack full task completion; >= 48 dp hit targets; KSL video on all explainer flows.

## 7. Integrity + anti-tamper

- **Per-record Ed25519 signature** binding voter biographic hash, kit_serial, clerk biometric ID, and HLC timestamp. Replayed or tampered records rejected by M02 on `eel_sequence` counter mismatch; `eel_tamper` event emitted.
- **Offline queue journal-before-act** ensures exact FSM resume after power failure at any transition. Proptest corpus covers arbitrary cut-points.
- **KitAttestation** (Android Play Integrity API verdict + TPM quote) sent on every boot; kit quarantined centrally within 60 s on mismatch via `services/biometric-edge-svc/crates/kit-attest/`.
- **SVID certificate revocation** propagated from SPIRE within 60 s; kit becomes inoperable; queue quarantined.
- **Geo-fence enforcement** via multi-constellation GNSS (GPS, GLONASS, Galileo) + IMU delta for spoof detection; enrolment blocked outside polygon; `eel_geo_violation` alert within 30 s.
- **Distress finger** triggers covert `eel.bve.distress_detected.v1` to back-end; UI identical to normal flow.
- **Honeypot mode** on revoked-kit login: sandbox served; attacker biometric silently captured.
- **Constant-time ABIS-lite comparison** in `packages/eel-biometric-rs` (EC-SEC-11).
- **PII log ban** via `eel_pii_tier_a` / `eel_pii_tier_b` regex; telemetry sink refuses classified fields; Semgrep enforced in CI.

## 8. Kenya statute alignment

- **Elections Act s.6A (public inspection + enrolment window)** -- gazetted window enforced by signed time-fence FSM; HSM-signed extension bundle required to reopen; forged extension rejected.
- **Elections Act s.14A (close time)** -- time-fence FSM honours HSM-signed close-time extensions.
- **Elections (Technology) Regulations 2017** -- kit attestation (TPM quote + firmware digest) logged to T1-audit on every boot; OTA images signed and A/B verified.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- stylus raw vectors discarded post-hash; only hash stored.
- **Kenya Data Protection Act 2019 s.31 (security)** -- per-field encryption, TLS 1.3 minimum, PQ-hybrid where supported (TLS 1.2 refused).

## 9. Honest caveats + external prerequisites

- FBI PIV IQS / STQC sensor certification timeline for the shortlisted vendor is unconfirmed; affects FR-M01-002. Deferred to hardware procurement gate (G3).
- Legacy Android feature matrix (degraded modalities on lowest-tier the Commission devices) requires sign-off from the Commission ICT before G3.
- ISO / FBI certification for on-device ABIS-lite matcher is shared risk with M03; both resolved together.
- SPIRE/SPIFFE deployment to all 65,000 kits is an infrastructure prerequisite (P04); kits fail closed without valid SVID.
- MDM (O01) channel for OTA distribution must be provisioned before field deployment; unsigned images refused at install.
- `packages/openapi-contracts/bvr/device-ingest.yaml` referenced in the spec does not exist on disk; the active contract is `packages/openapi-contracts/biometric-edge/openapi.yaml`.

## 10. Test coverage + operational runbooks

Test harness (`docs/plans/modules/bvr-device.md`, section "Test plan"):

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (field validators, MRZ parser, NFIQ2 gate, window FSM, distress matcher) | nextest + Flutter test | M01-UNIT- |
| Property (queue never drops on arbitrary power-cut; ABIS-lite no false accept) | proptest | M01-PROP- |
| Contract (bvr-backend ingest; VPRS handoff) | Pact | M01-PACT- |
| Integration (offline queue drain; geo-fence with boundary-svc double) | nextest + testcontainers | M01-INT- |
| E2E (full enrolment on emulator + real kit; legacy matrix) | patrol (Flutter) + HIL bench | M01-E2E- |
| Fuzz (MRZ parser; biographic field inputs) | cargo-fuzz | M01-FUZZ- |
| Load (12 enrolments/hr sustained 10 h) | k6 behind toxiproxy | M01-LOAD- |
| Chaos (power-cut FSM; cert revocation; OTA interrupt) | netem + kill -9 | M01-CHAOS- |
| Accessibility (WCAG 2.1 AA; TalkBack; 48 dp) | axe-core + manual TalkBack | M01-A11Y- |

Runbooks (to create): `docs/runbooks/biometric-edge-svc-degradation.md`, `docs/runbooks/bvr-device-legacy-android.md`.

Dashboard: `platform/observability/biometric-edge-svc/grafana-kit-health.json` (battery, connectivity, queue depth, enrolment rate, NFIQ2 distribution per station).

Full spec: `docs/plans/modules/bvr-device.md`. Service capabilities: `services/biometric-edge-svc/docs/CAPABILITIES.md`.
