# ADR-0026: Hardware Latitude Through Open-Source Drivers and Stable Abstractions

**Date**: 2026-08-22
**Status**: Accepted

## Context

The platform runs on heterogeneous kit hardware: biometric capture terminals,
RFID readers, HSMs, kit cameras, document scanners, GNSS/IMU modules, mobile
Android handsets, Tauri desktops, ballot printers, card manufacturers, RFID
tag suppliers, telco channels, and barcode scanners. The hardware prime (the
Mechrevo contract manufacturer) wants maximum latitude in selecting hardware
for each contract, so a reader or sensor from one vendor must be swappable for
an equivalent from another vendor without re-platforming any service.

Two facts drive the approach:

1. For many component classes a mature open-source driver stack exists
   (libfprint for fingerprints, sllurp/LLRP for UHF RFID, PKCS#11 for HSMs,
   bleak for BLE, OpenCV for imaging). Where such a stack exists, adopting it
   removes vendor SDKs, licence procurement, and lock-in, and it is the
   cheapest path to vendor-agnosticism because the standard the OSS stack
   speaks is usually the industry standard itself.
2. For component classes with no usable open-source driver, or where a vendor
   ships only a proprietary SDK, the platform must not hard-code that SDK.
   Instead it wraps the vendor interface behind a stable driver abstraction:
   a Rust `trait` plus an adapter/double pattern. Any vendor hardware that
   conforms to the relevant industry standard (or exposes the full featureset)
   plugs in behind the same trait without service changes.

This ADR complements ADR-0025 (which already converted RFID readers to sllurp
and bleak behind a shared adapter interface). It generalises that decision to
every hardware-touching component and records the full hardware matrix plus
each component's driver-abstraction contract.

## Decision

Prefer an open-source driver/hardware stack wherever one exists. Where none
exists, wrap the vendor interface behind a stable driver abstraction: a Rust
`trait` (the contract) with one concrete adapter per vendor and one synthetic
double for CI. Vendor selection for any component is then a build-time choice
of which adapter to bind, never a code change in a service.

Driver-abstraction conventions across the platform:

1. A `trait` defines the capture/operation contract for the component class.
   Trait methods return `Result<_, SensorError>` (or the integration's error
   type) so no vendor failure mode escapes the abstraction.
2. Each concrete vendor ships as an adapter implementing the trait. Adapters
   are thin: they translate the vendor SDK or protocol to the trait contract.
3. Every abstraction ships a synthetic double (mock) implementing the trait,
   used in CI and local dev so tests never require physical hardware.
4. Adapters live under `integrations/<domain>/adapter`, doubles under
   `integrations/<domain>/double`, and contract tests under
   `integrations/<domain>/contract-tests`, following the pattern established
   by `integrations/{printer-mes,telco-sms,telco-ussd,telco-vpn,card-manufacturer,hsm-pkcs11,mrz}`.
5. Where the OSS stack IS the industry standard (LLRP, PKCS#11), the trait
   mirrors that standard's operations and the adapter is a thin pass-through.

## Hardware Matrix

Each row: component | OSS driver stack (or none) | licence | standards/featuresets to support for vendor-agnostic selection | repo path(s).

| # | Component | OSS driver stack (or none) | Licence | Standards / featuresets to support | Repo path(s) |
|---|-----------|---------------------------|---------|-------------------------------------|--------------|
| 1 | Fingerprint sensors (optical, capacitive, multispectral) | libfprint via fprintd (capture) + SourceAFIS (feature extraction) | LGPL-2.1 (libfprint); MPL-2.0 (SourceAFIS) | ISO/IEC 19794-2 minutiae + CDEFFS extended features; NFIQ 2.2 quality (NIST SP 500-333); PAD L2 (ISO 30107-3); capture at 500 dpi for optical and capacitive, 1000 dpi for multispectral | `services/biometric-edge-svc/crates/sensor-driver/`, `services/abis-svc/crates/abis-fp-*`, `integrations/legacy-abis/` |
| 2 | Face sensors (RGB, NIR, 3D structured-light) | OpenCV (capture), InsightFace / ArcFace ONNX (embedding), OpenFace (fallback) | Apache-2.0 (OpenCV), MIT (InsightFace), BSD (OpenFace) | ISO/IEC 19794-5; FERET-aligned embedding; FRVT-style accuracy; NIR + RGB fusion; 3D depth map for structured-light vendors | `services/biometric-edge-svc/crates/sensor-driver/`, `services/abis-svc/crates/abis-face-*` |
| 3 | Iris sensors (near-infrared) | OpenCV for image intake; no OSS iris driver | Apache-2.0 (OpenCV) | ISO/IEC 19794-6; IREX-compliant probe image; NIR illumination; Daugman-style segmentation and Hamming-distance matching | `services/biometric-edge-svc/crates/sensor-driver/`, `services/abis-svc/crates/abis-iris-*` |
| 4 | UHF RFID readers (ballot-box covers) | sllurp (LLRP) for Impinj + Zebra; bleak (BLE) for CipherLab RS36 | MIT (both) | EPCglobal LLRP 1.1; EPC Gen2v2 Authenticate (OpSpec type 1024) with Gen2v1 fallback; ISO 18000-63 | `services/inventory-svc/py/rfid-reader-driver/`, `integrations/rfid-vendors/` (see ADR-0025) |
| 5 | HSMs | SoftHSMv2 (dev/synthetic) + cryptoki PKCS#11 v3.0 | BSD-2-Clause (SoftHSMv2); OASIS (PKCS#11) | PKCS#11 v3.0 profiles (CKM_RSA_PKCS, CKM_EC, AES, key generation, sign/verify, unwrap); vendor profiles for Thales, Entrust, Utimaco must map to the same CK_FUNCTION_LIST contract | `tools/softhsm/`, `integrations/hsm-pkcs11/adapter/eel-adapter-hsm-pkcs11-rs/`, `integrations/hsm-pkcs11/contract-tests/tests/vendor_profiles.rs` |
| 6 | Kit cameras / document scanners (MRZ per ICAO 9303) | OpenCV (image acquisition); MRZ parser behind the edge MRZ route | Apache-2.0 (OpenCV) | ICAO 9303 TD-1 / TD-3 MRZ; check-digit validation; OpenCV capture abstraction; camera-agnostic via trait | `services/biometric-edge-svc/crates/edge-api/src/routes/mrz.rs`, `integrations/mrz/` |
| 7 | GNSS / IMU / accelerometer on kits + mobile | Android sensors via platform channels; `sensors_plus` (Flutter) | Apache-2.0 (sensors_plus), AOSP | GPS spoof detection (M05); telemetry drop detection; NMEA 0183 output; IMU 6-axis (accel + gyro); fix accuracy and satellite count surfaced as telemetry | `services/field-mobile-svc/apps/*`, `services/ecc-svc/crates/telemetry-query/`, `services/ai-platform-svc/crates/telemetry-analytics/` |
| 8 | Mobile devices (Android, Tauri desktop) | Android AOSP; Play Integrity (Google, free); tpm2-tss for TPM | AOSP, Apache-2.0 | Play Integrity (Strong/Device/Basic verdicts); App Attest (iOS); TPM 2.0 quote (PCR measurements); KES-10k budget-device tolerance (lower-tier attestation with warning, not refusal) | `services/biometric-edge-svc/crates/kit-attest/`, `integrations/legacy-android-shim/`, `services/field-mobile-svc/apps/` |
| 9 | Ballot printers / MES | None (proprietary MES API) | proprietary | M39 ballot print order lifecycle; print order submit/status/verify; job barcode/QR tieback | `integrations/printer-mes/adapter/`, `integrations/printer-mes/double/` |
| 10 | Card manufacturers (voter cards) | None (vendor API) | proprietary | M53 voter-card production lifecycle; card order submit, personalisation status, dispatch; IAB smartcard personalisation handoff | `integrations/card-manufacturer/adapter/`, `integrations/card-manufacturer/double/` |
| 11 | RFID tag manufacturers (EPC Gen2v2) | None (supply-chain / procurement, no runtime driver) | n/a | EPC Gen2v2 (ISO 18000-63); EPC Tag Data Standard 2.0; tag TID memory; Authenticate support | `services/inventory-svc/py/rfid-reader-driver/` (reader side only) |
| 12 | Telco: SMS | smpplib / python-smpplib (SMPP) | MIT | SMPP 3.4; message submit/deliver; DLR (delivery receipts); concatenation; retry/backoff | `integrations/telco-sms/adapter/`, `integrations/telco-sms/double/` |
| 13 | Telco: USSD | None (telco gateway API) | proprietary | USSD session (menustring, session continue/end); short-code routing; vendor-agnostic via adapter | `integrations/telco-ussd/adapter/`, `integrations/telco-ussd/double/` |
| 14 | Telco: IPsec VPN | strongSwan / WireGuard | GPL-2.0 (strongSwan), GPL-2.0 (WireGuard) | IPsec IKEv2 tunnel; ESP; site-to-site to election data centres; MTU and NAT traversal | `integrations/telco-vpn/adapter/`, `integrations/telco-vpn/double/` |
| 15 | Barcode / QR scanners on kits | ZXing / ZBar | Apache-2.0 (ZXing), LGPL-2.1 (ZBar) | 1D (Code 128, EAN, UPC) + 2D (QR, DataMatrix) decode; ballot and package tracking; camera or laser scanner agnostic | `services/biometric-edge-svc/crates/sensor-driver/` (capture path) |
| 16 | IAB smartcards (PKCS#11, PIV) | OpenSC (PIV) + PKCS#11 | LGPL-2.1 (OpenSC); OASIS (PKCS#11) | NIST SP 800-73 PIV card applet; PKCS#11; ISO 7816; certificate + key storage; on-card biometric match for IAB cards | `integrations/hsm-pkcs11/` (shared cryptoki), `integrations/card-manufacturer/` |

## Driver-Abstraction Contracts

Per component, the contract that any vendor adapter must implement.

1. **Sensors (biometric and document)**. Trait `BiometricSensorDriver` in
   `services/biometric-edge-svc/crates/sensor-driver/src/sensor_driver.rs`.

   An adapter implements **one** required method,
   `capture(modality, timeout) -> Result<BiometricCapture, SensorError>`. Every
   per-modality method (`capture_fingerprint`, `capture_palm_vein`,
   `capture_live_photograph`, `capture_document_mrz`, and so on) is a provided
   wrapper over it. This was revised 2026-09-28: with fourteen modalities,
   requiring one method each would have made every new vendor adapter fourteen
   near-identical methods and turned any future modality into a breaking change
   for the whole fleet. One required method keeps vendor selection a build-time
   choice. An adapter that cannot serve a modality returns
   `SensorError::UnsupportedModality` rather than fabricating a sample;
   `supported_modalities` lets a client grey out a step it cannot run.

   Provided helpers: `quality_gate(quality)` and
   `capture_fingerprint_degraded(timeout)` for degraded-finger fallback.

   The `Modality` taxonomy covers eighteen captures: `fingerprint`,
   `finger_vein`, `palm_vein`, `wrist_vein`, `palm_print`, `face`,
   `live_photograph`, `periocular`, `iris`, `ear`, `retina`, `voiceprint`,
   `gait`, `signature`, `keystroke_dynamics`, `mouse_dynamics`, `dna`, and
   `document_mrz`. `Modality::is_biometric` is false only for `document_mrz`,
   whose output is document text rather than a template.

   Three axes beyond "what is captured" are modelled explicitly, because each
   changes how a client must behave:

   - **Collection mode.** `Active` modalities are acquired on demand within a
     timeout. `Passive` modalities (`keystroke_dynamics`, `mouse_dynamics`) are
     accumulated during a session and submitted at the end -- a timeout-bounded
     capture call is the wrong shape for them. `ExVivo` (`dna`) records a chain
     of custody rather than a timed acquisition. A client uses
     `Modality::collection_mode` to choose the right UI.
   - **Presentation-attack checking.** `capture_fingerprint_live` fails closed:
     an adapter with no declared liveness capability returns an error rather
     than an unverified sample, and there is deliberately no fallback to plain
     `capture_fingerprint`. A replica passes the quality gate and still fails
     liveness, so the two must never be substituted. The verdict records which
     technique was used (`blood_perfusion`, `subsurface_optical`,
     `challenge_response`, `thermal`), because a verdict without a method is
     not evidence of liveness.
   - **Behavioural vs physiological.** `Modality::is_behavioural` enumerates
     `voiceprint`, `gait`, `signature`, `keystroke_dynamics`, and
     `mouse_dynamics`, so a later consent and retention review can list the
     behavioural set from one place instead of auditing eighteen variants.

   `retina`, `keystroke_dynamics`, `mouse_dynamics`, `dna`, and `ear` have no
   binding ISO interchange standard. They are enumerated explicitly in
   `Modality::standard`'s tests rather than left implicit, because citing a
   standard that does not govern the data would be worse than citing none in a
   tender deliverable.

   Ships `MockSensorDriver` (synthetic double) with per-modality
   present/absent/timeout behaviour and a configurable supported set, so CI never
   needs physical hardware.

2. **RFID readers**. Shared LLRP client `LlrpCommonClient` plus `CipherLab` BLE
   adapter in `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/`.
   The interface is the adapter class contract (inventory, read, Gen2v2
   Authenticate, 3-state circuit breaker). See ADR-0025.

3. **HSMs**. Cryptoki `CK_FUNCTION_LIST` contract from
   `integrations/hsm-pkcs11/adapter/eel-adapter-hsm-pkcs11-rs/src/lib.rs`,
   with per-vendor profile tests in
   `integrations/hsm-pkcs11/contract-tests/tests/vendor_profiles.rs`. Vendor
   profiles (Thales, Entrust, Utimaco) map onto the same PKCS#11 contract;
   SoftHSMv2 at `tools/softhsm/` is the synthetic double.

4. **Kit cameras / document scanners (MRZ)**. Camera capture abstracted behind
   the biometric capture path; MRZ parse behind
   `services/biometric-edge-svc/crates/edge-api/src/routes/mrz.rs` calling
   `integrations/mrz/`. ICAO 9303 TD-1/TD-3 and check digits are the contract.

5. **GNSS/IMU**. Sensor read-out via `sensors_plus` platform channels;
   spoof and drop detection in the telemetry pipelines. The contract is the
   telemetry schema: fix accuracy, satellite count, and IMU axes surfaced as
   telemetry events.

6. **Mobile / desktop attestation**. `verify_attestation` in
   `services/biometric-edge-svc/crates/kit-attest/src/verify.rs` accepts
   Play Integrity, App Attest, TPM quote, or legacy payloads and returns an
   `AttestationVerdict` with tier + integrity level. Any device vendor must
   expose at least one of these paths.

7. **Printers / MES, card manufacturers, telco USSD**. FastAPI adapter +
   synthetic double per integration, following the
   `integrations/<domain>/{adapter,double,contract-tests}` pattern. The trait
   is the HTTP contract of the adapter (`submit`, `status`, `verify`).

8. **SMS (SMPP), barcode/QR, smartcards**. SMS uses the SMPP 3.4 standard via
   an OSS client; barcode/QR uses ZXing/ZBar decode; smartcards use OpenSC PIV
   and PKCS#11. These are standard-driven, so vendor selection follows the
   standard rather than a platform-specific trait.

## Naming

All identifiers use the neutral `eel` prefix. The vendor name `IEBC` appears
only in prose as the Kenyan elections body; no code, package, header, or event
identifier uses it. New adapters and crates continue the `eel-` prefix
(e.g. `eel-adapter-hsm-pkcs11-rs`, `sensor-driver`).

## Consequences

**Positive:**
- Hardware prime can bid any vendor for any component as long as it conforms
  to the listed standard or exposes the full featureset.
- No licence procurement for OSS-stack components; no vendor lock-in.
- CI runs on synthetic doubles, so tests never require physical hardware.
- A new vendor adapter is additive and isolated: one adapter crate, one
  contract test, no service changes.

**Negative:**
- OSS stacks cover capture and matching but not every vendor nuance (e.g.
  multispectral fingerprint optics, 3D structured-light depth maps); those
  surfaces need per-vendor adapters.
- Proprietary integrations (MES, card manufacturer, USSD gateway) still need
  the vendor's API; the abstraction confines that dependency to one adapter.
- More abstraction layers add indirection; mitigated by thin adapters and
  standard-mirroring traits.

## Alternatives Considered

1. **Standardise on one vendor per component**: Rejected. Directly contradicts
   the hardware prime's latitude requirement and creates procurement risk.

2. **Adopt OSS everywhere including where it is immature**: Rejected. For
   components with no mature OSS driver (MES, card manufacturer, USSD), a
   vendor wrapper behind a trait is more honest than forcing an immature
   stack.

3. **No abstraction, code to vendor SDK directly**: Rejected. Every vendor swap
   would become a code change and a re-platform.

## References

- ADR-0025 (`docs/architecture/decisions/0025-oss-rfid-stack.md`)
- libfprint / fprintd: https://fprint.freedesktop.org/ (LGPL-2.1)
- SourceAFIS: https://github.com/robertvazan/sourceafis-rs (MPL-2.0)
- sllurp: https://github.com/ransford/sllurp (MIT); bleak: https://github.com/hbldh/bleak (MIT)
- PKCS#11: https://docs.oasis-open.org/pkcs11/ (OASIS); SoftHSMv2 (BSD-2-Clause)
- OpenSC PIV: https://github.com/OpenSC/OpenSC (LGPL-2.1)
- ICAO 9303 MRZ: https://www.icao.int/publications/pages/publication.aspx?docnum=9303
- EPC Gen2v2 / ISO 18000-63: https://www.gs1.org/standards/epc-rfid/uhf-rfid-gen2 (royalty-free)
- SMPP 3.4, OpenCV, ZXing/ZBar: as cited in each matrix row
