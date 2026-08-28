# Deferral & Iris-Scanner Integration Plan

Status: authored 2026-08-27. Items marked **[DONE]** are implemented and verified
in this pass; the rest are sequenced with effort, dependency and blocker.

## A. Previously deferred conveniences

| # | Item | Priority | Approach | Effort | Blocker | Status |
|---|---|---|---|---|---|---|
| 1 | Declarative, config-driven recruitment pipeline | High | Make the recruitment gate set (documents/exam/interview/contract) configurable via `WORKFORCE_GATES` so a deployment can enable/disable gates; default = current behaviour | M | none | **[DONE]** |
| 2 | Duress-PIN flow through check-in | High | Wire `EelDuressHook` (kit-sdk) into the check-in flow so a duress PIN silently raises an alert while completing the check-in | M | none | **[DONE]** |
| 3 | Bulk document verification | High | `POST /v1/hr/documents/bulk-verdict` in support-admin payroll; dual-control still applies per item | M | none | **[DONE]** |
| 4 | Payslip view | Med | `GET /v1/payroll/payslips/{staff_id}` (list + line detail) in support-admin payroll; needs a payroll-runs/payslip store (currently an in-memory document store only) | M | payroll-runs store | deferred |
| 5 | Push-to-phone (FCM) | Med | Workforce notification delivery → FCM topic per worker | M | Firebase project + device token | deferred |
| 6 | SMS/WhatsApp status fallback | Med | Reuse field-mobile-svc USSD/WhatsApp adapters for posting/payment notifications | M | gateway credentials | deferred |
| 7 | Maps/directions to station | Low | Station geocode + deep-link to a maps provider | S | maps API key | deferred |
| 8 | In-app camera document capture | Low | Add `camera` plugin + capture sheet on the portal/mobile doc upload | M | none | deferred |
| 9 | 11-language parity | Med | Extend `hr-public` i18n dictionary beyond en/sw | L | translation review | deferred |
| 10 | Duress/panic surfacing (non-PIN) | Low | Discreet long-press action in the app shell | S | none | deferred |
| 11 | Report export (CSV/PDF) | Low | Readiness/audit export endpoint | M | none | deferred |

## B. Iris scanner (AntIris USB) integration

Device SDK extracted to `sdks/AntIris_Android_SDK_2.3.8/`. Public API (from the
sample app + `antiris.antusbcamera.jar`):

- `USBControl.getInstance()` — singleton controller.
- State probes: `isIdentify()`, `isEnrolling()`, `isIrexCapturing()`, `isPreviewing()`.
- Eye selection: `LEFT_EYE` / `RIGHT_EYE` / `TWO_EYES`.
- `antIrisEnroll(user, template, eye, timeoutMs, EnrollCallback)` — enrol.
- `antIrisIdentify(eye, IdentifyCallback)` — 1:N identify.
- `antIrisIrexCapture(user, eye, irexType, manual, timeoutMs, IrexCaptureCallback)` — ISO image capture.
- `antStopIdentify()`, `antStopEnroll(save)`, `antStopIrexCapture(save)`.
- `antStartStream()` / `antStopStream()`, `antGetStreamType(StreamType.STREAM_TYPE_IRIS)`.
- `Iris` data class (`getUser()`, template bytes); `IrisIso.generate(...)` for ISO 19794-6.

Phasing:

| Phase | Content | Verifiable? | Status |
|---|---|---|---|
| P1 | Dart `IrisScanner` abstraction + `PlatformChannelIrisTransport` + `SimulatedIrisTransport` + models + unit tests (mirrors the existing RFID reader pattern) | yes (`flutter analyze`/`test`) | **[DONE]** |
| P2 | Native Android bridge (`IrisPlugin.kt`) wrapping `USBControl` + JNI libs | needs device + Android toolchain | **[DONE]** — compiles + packages (`flutter build apk --debug`) |
| P3 | Wire capture into BVI/kit enrolment flow + ABIS template ingest (ISO 19794-6) | needs device for e2e | **[DONE client/bridge]** — `identify` accepts the enrolled gallery; `AbisKitClient` speaks the kit contract (`enrol`/`verify`/`gallery-config`); remaining: physical-device validation |

## D. Blocked placeholders (surfaced by the tech-debt audit)

These were inline `TODO`/stub sites. Each is now tracked here with its blocker,
and the code fails loudly (or clearly documents the placeholder) rather than
silently fabricating data.

| # | Item | Blocker | Resolution applied |
|---|---|---|---|
| D1 | `rts-icr` handwriting field extraction (`services/rts-svc/py/rts-icr/src/rts_icr/ocr.py`) | PaddleOCR ONNX handwriting model not vendored | `extract_fields` now raises `OcrUnavailableError` instead of returning synthetic 0.95-confidence fields; test updated to assert the error |
| D2 | Check-in biometric hash (`personnel/.../check_in_screen.dart`) | BVI kit pairing at the station (cross-device) | Comment updated to document the placeholder; hash will be injected by the dispatch flow once the app is paired with a kit |

## C. Definition of done

Every **[DONE]** item: compiles, passes its test suite, and `clippy`/`flutter
analyze` is clean. Hardware-dependent phases (P2/P3, push/SMS/maps/camera) are
blocked on credentials or physical devices and are explicitly documented, not
silently skipped.
