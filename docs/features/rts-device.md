# RTS device (M08)

Module M08 (`rts-device`) owned by `rts-svc`. Results Transmission System kit firmware: scan, sign, and transmit Form 34A within 90 s; SMS/USSD compressed fallback (CBOR+zstd) for connectivity-degraded polling stations; 7-day crash-consistent offline journal.

## Contents

1. What M08 does
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

## 1. What M08 does

At station close, the Presiding Officer uses the RTS kit app to scan the filled Form 34A (polling-station tally sheet), signs it with their biometric credential, and transmits to M10 (rts-backend). The entire scan-sign-transmit cycle must complete in <= 90 s. If data connectivity is unavailable, the app falls back to SMS/USSD with CBOR+zstd compression. If even SMS is unavailable, records queue locally for up to 7 days with crash-consistent journaling.

## 2. Data plane

Crates: `services/rts-svc/crates/rts-device-api/`, `rts-domain/src/aggregates/form34a.rs`.

| Aggregate | Purpose | Data class |
|---|---|---|
| **Form34AScan** | scan_id, station_code, image_hash, icr_confidence, vote_tallies (candidates x rows), presiding_officer_sig, kit_serial, ts_hlc | A |
| **TransmissionRecord** | transmission_id, scan_id, channel (DATA/SMS/USSD), status (QUEUED/SENT/ACKED), attempt_count, payload_size, compressed | C |
| **KitSessionEvent** | session_id, sealed_by (PO + Deputy), ts_hlc | C |
| **AuditEvent** | event_id, actor, action, ts_hlc, kit_serial | C |

Offline journal: encrypted SQLite WAL with signed-sequenced records; crash-consistent via journal-before-act.

## 3. Ingest sources

| Source | Notes |
|---|---|
| Flatbed scanner (kit-integrated) | Form 34A TIFF; ONNX handwriting ICR model runs on-device for pre-check |
| Camera backup | Kit rear camera if flatbed unavailable; perspective correction + deskew |
| Presiding Officer fingerprint | Biometric signature binding PO identity to Form 34A scan |
| NTP / GNSS | Timestamp anchor; HLC used for all records |

## 4. Pipeline

Crates: `services/rts-svc/crates/rts-device-api/src/`, Flutter app under `services/rts-svc/apps/`.

```
PO biometric login (fingerprint, rate-limited PIN fallback)
      -> time-fence FSM: station close confirmed (dual-control seal)
      -> scan Form 34A:
             flatbed TIFF -> deskew + perspective correct
             on-device ONNX ICR: extract vote tallies per candidate row
             ICR confidence displayed to PO; low-confidence cells flagged
      -> PO reviews extracted tallies against physical form
      -> PO edits any ICR error (with mandatory reason note)
      -> PO biometric-signs Form34AScan:
             Ed25519 binding: scan_image_hash + tallies + PO_identity
                              + station_code + kit_serial + HLC
      -> attempt primary transmission (HTTPS gRPC to rts-device-api):
             success -> TransmissionRecord.status = ACKED
             fail    -> fallback channel selection:
                          3G/2G SMS: CBOR+zstd payload (<= 160 chars/segment)
                          USSD: structured USSD session with CBOR+zstd
                          both fail -> local journal queue (7-day capacity)
      -> emit eel.rts.form34a_transmitted.v1 (on success)
             eel.rts.form34a_queued.v1 (on journal)
      -> ECC notified of transmission status within 30 s
```

ICR model: ONNX model served via `rts-svc/crates/rts-device-api/src/codec.rs`; model runs on ARM CPU (no GPU required); confidence threshold configurable. Algorithm: CNN-based offline handwriting recognition (Graves, Schmidhuber, IEEE ICDAR 2009 for CTC-based recognition family).

SMS fallback: CBOR (RFC 7049) + zstd (Collet et al. 2016, Meta open-source) compression; fuzz-tested via `crates/rts-device-api/fuzz/fuzz_targets/fuzz_sms_decode.rs`.

## 5. API surface

Contract: `packages/openapi-contracts/rts/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /rts/v1/device/form34a` (gRPC) | Kit (SVID mTLS) | Submit signed Form 34A scan + tallies |
| `POST /rts/v1/device/session/seal` (gRPC) | Kit (SVID mTLS) | Submit dual-control session-close event |
| `GET /rts/v1/device/status/{station_code}` (gRPC) | Kit (SVID mTLS) | Check M10 receipt confirmation |
| `POST /rts/v1/device/attestations` (gRPC) | Kit (SVID mTLS) | Boot-time TPM quote |

SMS/USSD fallback channel: `POST /ussd/v1/rts` on USSD gateway; CBOR+zstd payload decoded by `crates/rts-device-api/src/codec.rs`.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| RTS kit app | `services/rts-svc/apps/` (Flutter) | PO tally review, ICR correction, biometric sign, transmit |
| ECC status panel (M16) | ecc-svc consuming `eel.rts.form34a_transmitted.v1` | Real-time transmission status per station |

Languages: English + Kiswahili. WCAG 2.1 AA; audio guidance. ICR review screen shows form image alongside extracted tallies; PO corrects cell-by-cell.

## 7. Integrity + anti-tamper

- **Ed25519 signature** on every Form34AScan binding image hash, tallies, PO identity, station code, kit_serial, HLC; tampered records rejected by M10 on signature verification.
- **Image hash** (SHA-256 of TIFF) stored in signature; any post-scan image manipulation detectable.
- **CBOR+zstd fallback integrity** -- CBOR payload includes the same Ed25519 signature; fuzz corpus (`fuzz_sms_decode.rs`) guards parser.
- **Offline journal journal-before-act** -- crash-consistent; proptest covers arbitrary power-cut.
- **Kit attestation** -- TPM quote on every boot; kit quarantined centrally within 60 s on mismatch.
- **Dual-control session seal** -- station close requires PO + Deputy biometric; KitSessionEvent carries both identities.
- **ICR confidence audit** -- ICR confidence per cell logged alongside final tallies; low-confidence cells require PO override with reason note.
- **7-day queue with signed-sequenced records** -- gap audit run on reconnect; zero record loss guaranteed by property test.
- **Replay protection** -- `eel_idempotency_key` per Form34AScan; server returns cached 200 on duplicate.

## 8. Kenya statute alignment

- **Elections Act s.39 (tally declaration)** -- Form 34A is the legal declaration instrument; M08 digitises and transmits it while preserving the original paper form.
- **Elections (General) Regulations Reg. 83 (form signing)** -- PO biometric-signs Form 34A before transmission; signature verifiable by M10 and court.
- **Elections (Technology) Regulations 2017** -- signed image hash + transmission log provides the electronic audit trail; KEBS-certifiable evidence chain.
- **Kenya Data Protection Act 2019 s.31** -- encrypted local journal; TLS 1.3 on all data-path transmission.

## 9. Honest caveats + external prerequisites

- **ICR model accuracy** on handwritten Kenyan forms must be validated on a representative field corpus before G3; current model is ONNX placeholder awaiting fine-tuning on the Commission form samples.
- **SMS/USSD gateway commercial arrangement** (Safaricom / Africa's Touching) required; adapter written; live gateway allocation pending.
- **90 s SLO** requires certified scanner hardware; scanner vendor and model must be finalised before hardware procurement gate.
- **7-day offline capacity** assumes peak Form 34A volume (one form per station per election); actual capacity far exceeds this for normal elections but must be verified for multi-ballot days.
- Form 34A PDF paper layout is the canonical legal instrument; the digital tally is legally subordinate until the Elections Act is amended.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (ICR confidence gate, CBOR+zstd codec, Ed25519 sign/verify, time-fence FSM) | nextest + Flutter test | M08-UNIT- |
| Property (queue no-loss on arbitrary power-cut; replay dedup; SMS decode round-trip) | proptest | M08-PROP- |
| Contract (M10 form34a ingest; ECC status consumer) | Pact | M08-PACT- |
| Fuzz (SMS CBOR+zstd decoder; image scanner parser) | cargo-fuzz (`fuzz_sms_decode.rs`) | M08-FUZZ- |
| Integration (USSD gateway double; M10 double; kit HIL) | nextest + testcontainers | M08-INT- |
| Load (90 s scan-sign-transmit on 1,000 concurrent kits; SMS fallback throughput) | k6 | M08-LOAD- |
| Chaos (data link blackhole; SMS gateway down; power-cut mid-scan) | netem + kill -9 | M08-CHAOS- |

Runbook (to create): `docs/runbooks/rts-svc-kit-degradation.md` (SMS fallback activation; offline-queue overflow; ICR failure escalation).

Full spec: `docs/plans/modules/rts-device.md`. Crate: `services/rts-svc/crates/rts-device-api/`. Fuzz target: `services/rts-svc/crates/rts-device-api/fuzz/fuzz_targets/fuzz_sms_decode.rs`.
