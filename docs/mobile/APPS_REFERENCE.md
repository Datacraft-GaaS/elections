# Mobile Apps — Complete Reference (14 apps)

## 1. Poll Clerk (M31) — poll-clerk.md (167 lines, expanded to 350+)
- **Purpose**: Primary polling-station field tool for Presiding Officer
- **Functions**: MFA sign-in (FR-1), reference library offline (F2), station dashboard WebSocket (F3), voter lookup fallback with DPO attestation (F4), priority queue management (F5), structured incident reporting 7 categories (F6), paper-backup wizard (F7), kiosk/BVV assist (F8), EOD dual-signature report (F9), security envelope mTLS/FLAG_SECURE (F10), battery discipline (F11)
- **Capabilities**: 11 feature screens, 27 lib files, 11 locales, token-bucket rate limit, EWMA ETA, BM25 FTS5 search, Ed25519 signatures, UUID7 idempotency
- **Architecture**: M2 completed (station_dashboard, incident, queue, lookup, eod_report, reference_library, kiosk — all with eel_design), M3 drivers gated, M4 contract bound

## 2. Observer (M34) — observer.md (121 lines → complete)
- **Purpose**: Independent observer accreditation + station visit + tally verification
- **Functions**: Accreditation (F1), MFA FIDO2/biometric (F2), category-gated feature access via SpiceDB (F3), portal results (national/county/constituency/station, F4), Merkle proof retrieval (F5), root-hash pinning + auto-tampering incident (F6), station bookmarks (F7), bulk constituency download (F8), GPS check-in + visit log (F9), session sign-off (F10), structured incidents (F11), sampling (seeded selection, F12), verification checklist (F13), countersignature (F14), EOM report (F15), offline cache + stale-since (F16), systemic-incident flag exact-once (F17), security envelope (F18), locales EN/SW (F19), WAL integrity (F20)
- **Features**: auth/login, status (rewritten), mission_export, station_visit, tally_verify, incident
- **Design**: EelScaffold + EelStatusChip + EelResultCard + EelSealStamp on login

## 3. Party Agent (M32) — party-agent.md (135 lines → complete)
- **Purpose**: Party agent check-in, rollup, dispute, countersign, forms
- **Functions**: Dashboard with state chips + button groups (F1), check-in with QR/biometric (F2), rollup aggregation (F3), dispute filing with evidence (F4), duress alert (F5), countersignature chain (F6), forms library (F7), incident reporting (F8), QR verification (F9)
- **Screens**: dashboard (rewritten EelScaffold), check_in, rollup, dispute, duress_alert, countersign, forms, incidents, qr_verify, auth/login
- **Design**: EelStatusChip, EelButton, EelChips, EelResultCard throughout

## 4. Citizen Verify (M13) — citizen-verify.md (102 lines → complete)
- **Purpose**: Citizen-facing verification (register lookup, station finder, results, bookmark)
- **Functions**: Onboarding (F1), station finder (F2), results browser (F3), register lookup with DPO attestation + justification (F10–12), bookmark save (F8), incident report anonymous (F7)
- **Screens**: onboarding (EelScaffold/EelButton/EelResultCard/EelSealStamp), otp, register/lookup, station_finder, results, bookmarks, incident/report, station_finder
- **Design**: EelTextField, EelButton, EelBanner, EelChips, EelStatusChip, EelSealStamp

## 5. Field Support (M29) — field-support.md (134 lines → complete)
- **Purpose**: Field technician support (warranty, diagnostics, shift report, ticket queue)
- **Functions**: Shift report (F1, rewritten EelScaffold/EelResultCard), diagnostics (F2), incident (F3), warranty (F4), ticket_queue (F5, new — EelStatusChip, EelButton, EelResultCard)
- **Screens**: shift_report, diagnostics, incident, warranty, ticket_queue

## 6. Personnel (M29) — personnel.md (117 lines → complete)
- **Purpose**: Personnel management (assignment, check-in/check-out, status ping, incident)
- **Functions**: Status ping (F1, EelResultCard/EelButton), assignment (F2, EelScaffold/EelStatusChip), check_in (F3, EelTextField), check_out (F4, EelBanner), incident (F5, EelTextField/EelButton)
- **Screens**: status_ping, assignment, check_in, check_out, incident

## 7. RTS Kit (M28) — rts-kit.md (98 lines → complete)
- **Purpose**: Real-time transmission (RFID scan, GSM modem SMS, AT commands)
- **Functions**: RFID scan/transmit (F1), AT command (F2: AT+CMGF, AT+CMGS), GsmModem lifecycle (F3), SMS PDU encoder (sms_pdu.dart), USB serial transport (usb_serial_transport.dart), session (F4)
- **Features**: scan_screen, scan_result_screen (rewritten), rts_workflow/sign/transmit, at_command/gsm_modem
- **Design**: EelScaffold + EelButton + EelResultCard for scan results

## 8. BVR Kit (M0–M3) — bvr-kit.md (130 lines → complete)
- **Purpose**: Biometric voter registration (FAP60 fingerprint)
- **Functions**: FAP60 fingerprint capture (Fap60Driver via MethodChannel 'eel.bvr/fingerprint', LIVESCAN_Init + captureFinger), MRZ scan (F3 — document camera SDK not present), capability status (F4), M3 feature gate (F5), session (F6)
- **Hardware-gated**: returns FingerprintResult.invalid() when plugin not registered; never crashes on missing device

## 9. BVI Kit (M3) — bvi-kit.md (114 lines → complete)
- **Purpose**: Biometric voter identification (AntIris iris)
- **Functions**: AntIris iris capture (AntIrisDriver via USBControl.antIrisIrexCapture + IrisIso.generate), identification workflow (F2), session (F3), capability status (F4), M3 feature gate (F5)
- **SDK**: USBControl.jar / lib_Antiris.so — exact signatures in M3_HARDWARE_INTEGRATION.md

## 10. Diaspora Kit (M30) — diaspora-kit.md (95 lines → complete)
- **Purpose**: Diaspora polling station (station screen, pouch, Form 34A)
- **Functions**: Station overview (F1), sealed pouch contents (F2), Form 34A verification (F3)
- **Screens**: station_screen, pouch_screen, form34a_screen (all rewritten EelScaffold/EelResultCard/EelStatusChip/EelButton)
- **Integration**: BVI back-end, M3 session, diaspora_driver

## 11. LMS Offline (training) — lms-offline.md (71 lines → complete)
- **Purpose**: Offline training module assessment (course download, assessment, sync)
- **Functions**: Module repository (cached/offline, 5-module limit), assessment screen (F2), modules list (F1, rewritten with EelScaffold/EelResultCard), sync (F3), assessment path fixed to /v1/training/lms/assessments/{id}/submissions
- **Design**: EelScaffold, EelResultCard, EelButton, EelStatusChip, EelLoadingState for all states

## 12. Accred Scanner (training) — accred-scanner.md (71 lines → complete)
- **Purpose**: Accreditation badge scanning with CRL verification (CA-verified)
- **Functions**: QR/barcode scan (MobileScanner), CRL endpoint /v1/security/crl (CA verification), scan result (revoked/active/suspended status indicator), badge verification via JWT claim 'sub'
- **Screens**: scan_screen (preserves MobileScanner + JWT parsing logic), scan_result_screen (rewritten EelScaffold + EelResultCard + EelBanner for CRL stale + zones)
- **Fix**: endpoint corrected from /v1/crl to /v1/security/crl

## 13. Legacy Android (M29) — legacy-android.md (29 lines, expanded)
- **Purpose**: Backward compatibility for deprecated Android devices
- **Functions**: Deep-link to kit screens (eel://kit/bvr/enrol, eel://kit/bvi/identify), MDM install-guidance fallback, legacy_home.dart, PARITY.md, url_launcher dependency
- **State**: shim fixed, attestation_tier=legacy marked deprecated

## 14. Hello Flutter (M0) — hello-flutter.md (17 lines, expanded)
- **Purpose**: Baseline validation / build verification
- **Functions**: Basic Flutter scaffold, build.gradle.kts (AGP 8.9.1, Kotlin 2.0.21), AndroidManifest.xml, resources, v2 embedding, APK output verified 143MB
- **Validation**: proves Java 17 + Gradle 8.11.1 + AGP 8.9.1 + Kotlin 2.0.21 stack works; sequential build verified; design-system import verified via package path (`packages/design-system/flutter/`)

## 15. Kit L1 (M4) — kit-l1.md (38 lines → complete)
- **Purpose**: Kit-level anomaly detection pipeline
- **Functions**: KitL1Pipeline wrapping AnomalyDetector, integration with biometric service
- **Design**: EelScaffold + EelButton in integration layer

---

## Shared design-system usage (all apps)

Every screen across all 14 apps uses the `eel_design` package (`packages/design-system/flutter/lib/src/widgets/`):
- `EelScaffold` (fixed/scrollable variants, padding, title)
- `EelButton` (primary/tertiary/outlined, fullWidth, loading state, icon)
- `EelResultCard` (leading icon + title + subtitle, elevation, color tokens)
- `EelTextField` (label, hint, maxLines, obscureText, error state)
- `EelStatusChip` (ok/warning/critical/info, severity color token)
- `EelBanner` (message + severity, info/warning/critical)
- `EelChips` (filter/action chip group, selection state)
- `EelSealStamp` (trust indicator for authentication/login screens)
- `EelSecureScreen` (FLAG_SECURE wrapper for sensitive interfaces)
- `EelTextStyle` (displayMedium, titleLarge, titleMedium, bodyMedium, bodySmall, caption)
- `EelColors.of(context)` (design token extraction from theme extension)
- `EelLoadingState` (spinner + label, accessibility live-region)
- `EelEmptyState` / `EelErrorState` (first-class empty/error UI patterns)
- `EelStepFlow` (wizard/progress rail for multi-step workflows)
- `EelSyncIndicator` (last-sync timestamp + status chip)

Every app uses `EelScaffold` (no bare `Scaffold` remains); every button is `EelButton`; every info card is `EelResultCard`; every form uses `EelTextField`; all severity states use `EelStatusChip`; contextual messages use `EelBanner`; authentication screens include `EelSealStamp`; all texts use `EelTextStyle`; no `Theme.of(context)` direct use remains for typography or colors.

---

## Build / deployment

- **Sequential** (`build-all-apks-sequential.sh`): absolute paths, kills Gradle daemon between apps (`gradlew --stop` + registry delete), 2GB heap (`Xmx2G`), one at a time (~28 min for 14 apps)
- **Parallel** (`build-all-apks.sh`): 2 concurrent, same config, JSON summary output
- **Verified APKs**: hello-flutter 143MB (new), bvi-kit 166MB (pre-existing Aug 27)
- **Java/Gradle stack**: openjdk@17 (brew), `JAVA_HOME`, `flutter config --jdk-dir`, Gradle 8.11.1, AGP 8.9.1, Kotlin 2.0.21, `build.gradle.kts` everywhere, `gradle-wrapper.properties` uniform, `local.properties` → `/opt/homebrew/share/flutter`
