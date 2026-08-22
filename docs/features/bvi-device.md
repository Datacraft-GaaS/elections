# BVI device (M05)

Module M05 (`bvi-device`) owned by `biometric-edge-svc`. Kit-side polling-day voter identification: 1:1 multi-modal biometric authentication against a locally cached register extract, with GNSS + IMU spoof checks, a time-fence FSM, dual-control close, and the Elections Act s.44A fallback path.

## Contents

1. What M05 does
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

## 1. What M05 does

On polling day, every voter presents at a polling station. The clerk scans the voter's ID / QR barcode; the BVI kit app performs a 1:1 biometric match against the voter's cached template (fingerprint primary, iris / face fallback). A successful match marks the voter as identified in the local event log; the event is submitted to M06 (bvi-backend) for the national turnout rollup. M05 never contacts M03 (ABIS) live on polling day; all matching is local.

## 2. Data plane

Crates: `services/biometric-edge-svc/crates/edge-api/src/routes/identification.rs`. Kit-side store: encrypted SQLite WAL queue (same journal as M01).

| Aggregate | Purpose | Data class |
|---|---|---|
| **IdentificationEvent** | event_id, voter_id, station_code, modality_used, match_score, auth_outcome (PASS/FAIL/S44A), ts_hlc, kit_serial, clerk_id | A |
| **RegisterExtract** | voter_id, biographic_digest, template_ref, polling_station_code | A (read-only, cached) |
| **KitSessionEvent** | session_id, opened_by, sealed_by, dual_control_officers, sealed_at, ts | C |
| **AuditEvent** | event_id, actor, action, ts_hlc, kit_serial | C |

Register extract pre-loaded to each kit before polling day; extract is station-scoped and field-encrypted.

## 3. Ingest sources

| Source | Notes |
|---|---|
| Voter ID / QR barcode scan | QR payload signed with Ed25519 (bvr-backend); kit verifies before lookup |
| Fingerprint sensor (1:1 match) | ISO/IEC 19794-2 probe; matched against cached template |
| Iris camera (fallback) | ISO/IEC 19794-6; used when fingerprint quality < threshold or s.44A invoked |
| Face camera (fallback) | ICAO 9303 / ISO 19794-5; used when both fingerprint and iris unavailable |
| GNSS + IMU | Multi-constellation (GPS, GLONASS, Galileo) + IMU delta for geo-fence and spoof detection |
| NTP / GNSS | Time-fence FSM reference; polling-window open/close enforced |

## 4. Pipeline

Crates: `services/biometric-edge-svc/apps/bvi-kit/` (Flutter), `crates/edge-api/src/routes/identification.rs`.

```
clerk biometric login (fingerprint / PIN rate-limited fallback)
      -> time-fence FSM: polling window open?
             no  -> kit locked; display "polls closed" screen
             yes -> proceed
      -> geo-fence check (GNSS + IMU): kit inside station polygon?
             no  -> block; eel_geo_violation alert within 30 s
      -> voter ID / QR scan -> verify Ed25519 signature on QR payload
      -> lookup voter in local register extract (station-scoped)
             not found -> display "not registered at this station"
      -> probe biometric capture (fingerprint primary):
             NFIQ 2.2 quality gate; 3 attempts before fallback
      -> 1:1 match against cached template (constant-time compare)
             PASS (score >= threshold):
               record IdentificationEvent.auth_outcome = PASS
             FAIL (3 attempts):
               offer s.44A face-and-ID path (Elections Act s.44A)
                 clerk invokes s.44A -> auth_outcome = S44A
                 manual register entry; supervisor countersign required
      -> mark voter as identified in local SQLite journal
      -> sign IdentificationEvent (Ed25519, kit_serial, HLC)
      -> append to encrypted offline queue
      -> sync to M06 bvi-backend when online
             eel.bvi.identification_event.v1 (Kafka eel.bvi.v1)
      -> dual-control close at station close:
             Presiding Officer + Deputy both biometric-auth
             -> KitSessionEvent.sealed_by = [PO, Deputy]
             -> emit eel.bvi.session_sealed.v1
```

Throughput SLO: identification cycle <= 30 s per voter. Offline-first: 7-day queue capacity (shared with M01).

## 5. API surface

Contract: `packages/openapi-contracts/biometric-edge/openapi.yaml` (shared with M01).

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /bvi/v1/identification` (gRPC) | Kit (SVID mTLS) | Submit signed IdentificationEvent batch to M06 |
| `POST /bvi/v1/session/seal` (gRPC) | Kit (SVID mTLS) | Submit dual-control KitSessionEvent on close |
| `GET /bvi/v1/register-extract` (gRPC) | Kit (SVID mTLS) | Download station-scoped register extract before polling day |
| `POST /bvr/v1/attestations` (gRPC) | Kit (SVID mTLS) | Boot-time TPM quote (shared with M01) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| BVI kit app | `services/biometric-edge-svc/apps/bvi-kit/` (Flutter) | Polling-day identification UI for ~65k new kits |
| Legacy Android shim | Same Flutter codebase + degradation layer | ~14,100 legacy devices |

Languages: English + Kiswahili; WCAG 2.1 AA; audio guidance; >= 48 dp hit targets; KSL video on s.44A explainer flow. Voter-facing feedback limited to PASS / FAIL; no score displayed.

## 7. Integrity + anti-tamper

- **QR signature verification** -- QR payload signed by bvr-backend (Ed25519); forged or replayed QRs rejected.
- **Constant-time 1:1 match** -- `packages/eel-biometric-rs`; timing-side-channel protected (EC-SEC-11).
- **GNSS + IMU spoof detection** -- multi-constellation + IMU delta; geo-fence violation blocks identification and sends `eel_geo_violation` alert within 30 s (EC-KIT-01).
- **Time-fence FSM** -- polling window enforced by gazetted open/close times; HSM-signed extension bundle required to extend; forged extension rejected.
- **Dual-control session close** -- both Presiding Officer and Deputy must biometric-auth to seal; `KitSessionEvent` carries both identities.
- **Per-event Ed25519 signature** binding voter_id, station_code, modality, auth_outcome, kit_serial, HLC; tampered events detected by M06 on counter mismatch.
- **Offline queue journal-before-act** -- property-tested against arbitrary power-cut sequences.
- **Kit attestation** -- TPM quote on every boot; kit quarantined centrally within 60 s on mismatch.
- **Register extract integrity** -- extract signed by bvr-backend before push; kit verifies before loading into SQLite.

## 8. Kenya statute alignment

- **Elections Act s.44A (assisted identification)** -- face-and-ID path built-in; supervisor countersign required; `auth_outcome = S44A` recorded; no voter denied participation.
- **Elections Act s.14A (polling hours)** -- time-fence FSM; HSM-signed extensions only.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- station-scoped register extract only; national register never on kit; biometric templates purged after polling day (configurable retention policy).
- **Kenya Data Protection Act 2019 s.31 (security)** -- field-encrypted extract; TLS 1.3 on sync; kit-side encrypted SQLite.
- **Elections (Technology) Regulations 2017** -- kit attestation logged; signed events submitted to transparency log via M06.

## 9. Honest caveats + external prerequisites

- **Register extract pre-load SLA** -- extract for all 46k+ polling stations must reach kits before polls open; this is an operations prerequisite, not a code gap.
- **Legacy Android feature matrix** -- biometric modality degradation on lowest-tier IEBC devices needs IEBC ICT sign-off before G3 (shared risk with M01).
- **1:1 match threshold calibration** -- operating threshold must be set by IEBC against local population FAR/FRR data; ABIS FAR/FRR reports from M03 feed this calibration.
- **s.44A manual path** -- supervisor countersign requires supervisor to be physically present; ECC remote co-sign (EC-OPS-04) is a fallback but requires connectivity.
- Kit GPS + IMU spoofing in dense urban environments: alert rate may need threshold tuning in field trials.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (time-fence FSM, QR sig verify, GNSS spoof detect, dual-control seal) | nextest + Flutter test | M05-UNIT- |
| Property (queue never drops on power-cut; no double-identification) | proptest | M05-PROP- |
| Contract (M06 identification ingest; register extract download) | Pact | M05-PACT- |
| Integration (register extract push; M06 ingest double) | nextest + testcontainers | M05-INT- |
| E2E (full polling-day flow on emulator + HIL bench; s.44A path) | patrol (Flutter) + HIL | M05-E2E- |
| Chaos (power-cut at every FSM state; cert revocation; dual-control partial failure) | netem + kill -9 | M05-CHAOS- |
| Accessibility (WCAG 2.1 AA; TalkBack; 48 dp; KSL s.44A flow) | axe-core + manual | M05-A11Y- |

Runbooks (shared with M01): `docs/runbooks/biometric-edge-svc-degradation.md`.

Full spec: `docs/plans/modules/bvi-device.md`. Service capabilities: `services/biometric-edge-svc/docs/CAPABILITIES.md`.
