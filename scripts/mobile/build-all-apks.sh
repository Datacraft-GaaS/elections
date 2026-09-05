#!/usr/bin/env bash
# scripts/mobile/build-all-apks.sh — comprehensive APK build for all mobile apps.
#
# =============================================================================
# USAGE
# =============================================================================
#   scripts/mobile/build-all-apks.sh [options]
#
# EXAMPLES
# --------
#   # Debug build of every app (fastest — no signing, no analyse, no tests).
#   scripts/mobile/build-all-apks.sh
#
#   # Release build with all quality gates (analyse + tests must pass first).
#   scripts/mobile/build-all-apks.sh --release --analyze --test
#
#   # Build only citizen-verify (useful while iterating on that app).
#   scripts/mobile/build-all-apks.sh --app=citizen-verify
#
#   # Skip the two hardware-gated apps (FAP60 / AntIris not present).
#   scripts/mobile/build-all-apks.sh --skip=bvr-kit,bvi-kit
#
#   # Parallel build (2 apps at once) for faster wall-clock time.
#   scripts/mobile/build-all-apks.sh --parallel
#
#   # Clean before building (removes build/ directories first).
#   scripts/mobile/build-all-apks.sh --clean --release
#
#   # Sign a release build with a Java keystore.
#   scripts/mobile/build-all-apks.sh --release \
#     --keystore=keystore.jks \
#     --keystore-pass=secret \
#     --key-alias=eel-key \
#     --sign
#
#   # Combine flags.
#   scripts/mobile/build-all-apks.sh --release --analyze --test \
#     --skip=bvr-kit,bvi-kit --parallel
#
#   # Show this help.
#   scripts/mobile/build-all-apks.sh --help
#
# =============================================================================
# BEHAVIOUR
# =============================================================================
#
# Apps built (14 total — per plan.md §34, preserving all 17 apps; 3 are KMP/desktop
# and do not have an Android target, so they are excluded):
#
#   Field-mobile-svc (persona apps):
#     poll-clerk        — Election official workstation app (station dashboard, queue,
#                        voter lookup, reference library, EOD report, kiosk mode).
#     observer         — Election observer app (status, mission export, tally
#                        verify, station visit, incident report).
#     party-agent      — Party agent app (check-in, dashboard, rollup, dispute,
#                        duress alert, countersign, forms, incidents, QR verify).
#     field-support    — Field support technician app (shift report, diagnostics,
#                        check-in, status ping, incident).
#     personnel        — Personnel management app (check-in, status ping).
#     citizen-verify   — Public-facing app (OTP login, register lookup, station
#                        finder, results browser with Merkle proof, incident
#                        report, bookmarks). Created M4.
#
#   Biometric-edge-svc (kit apps):
#     bvr-kit          — Biometric voter registration kit (FAP60 fingerprint +
#                        MRZ, offline queue). **Hardware-gated**: FAP60 USB scanner
#                        must be present or `FingerprintResult.invalid()` is
#                        returned gracefully.
#     bvi-kit          — Biometric verification identification kit (AntIris iris
#                        + FAP60 fingerprint). **Hardware-gated**: AntIris USB
#                        camera must be present or `IrisResult.invalid()` is
#                        returned gracefully.
#     diaspora-kit     — Diaspora voting kit (BVR + BVI combined).
#     legacy-android   — Shim (WP-3.8). Deep-links into bvr-kit / bvi-kit or
#                        shows MDM managed-app store install guidance.
#
#   Rts-svc (kit app):
#     rts-kit          — Results transmission kit (RTS). Includes GsmModem for
#                        SMS fallback via `usb_serial` + AT commands (3GPP
#                        TS 27.005/27.007). GSM modem must be present.
#
#   Training-svc (utility apps):
#     lms-offline      — Offline learning management system (assessment + modules).
#                        API paths fixed to /v1/training/lms/... (M4).
#     accred-scanner   — Accreditation scanner (CRL endpoint: /v1/security/crl,
#                        M4).
#     hello-flutter    — Minimal CI validator app.
#
#   Excluded (KMP/desktop, no Android target):
#     ro-desktop       — Registration officer desktop (Kotlin Multiplatform).
#     kit-l1           — Kit L1 anomaly detection (Kotlin Multiplatform).
#
# Output locations:
#   build/apks/<app>-<mode>.apk   # APKs copied here with stable names
#   build/apks/summary.json       # Machine-readable result summary
#   build/apks/build.log          # Full concatenated build log
#   build/apks/logs/<app>.log     # Per-app log (useful when debugging failures)
#
# Exit codes:
#   0   All requested builds succeeded.
#   1   One or more builds failed, or --analyze / --test gates failed.
#   2   Invalid arguments.
#
# =============================================================================
# HARDWARE GATES (M3 per plan.md §11.0)
# =============================================================================
#
# bvr-kit: biometric capture calls Fap60Driver via MethodChannel.
#   SDK: FUFingerSDK.LIVESCAN_Init() + captureFinger() (FingerUp/FUFingerSDK).
#   Native: Fap60Plugin.kt registers com.eel.biometric/fap60 channel.
#   Without FAP60 hardware: driver returns FingerprintResult.invalid() — no crash.
#
# bvi-kit: iris capture calls AntIrisDriver via MethodChannel.
#   SDK: USBControl.antIrisIrexCapture() + IrisIso.generate() (AntIris).
#   Native: AntIrisPlugin.kt registers com.eel.biometric/antiris channel.
#   Without AntIris hardware: driver returns IrisResult.invalid() — no crash.
#
# rts-kit: SMS transmission calls GsmModem via usb_serial + AT commands.
#   AT: AT+CMGF (text/PDU mode), AT+CMGS (send), AT+CMSS (send from storage).
#   PDU encoder: sms_pdu.dart (3GPP 23.038 CBS DCS, 23.040 TPDU).
#   Without GSM modem: GsmModem returns SendResult.queued() for later retry.
#
# =============================================================================
# APK SIGNING
# =============================================================================
#
# Debug APKs are signed with the default Flutter debug keystore (no action needed).
#
# Release APKs are unsigned by default (unsigned APKs cannot be installed on
# non-rooted devices). To sign them:
#
#   1. Create a keystore:
#      keytool -genkey -v -keystore keystore.jks \
#        -alias eel-key -keyalg RSA -keysize 2048 -validity 10000 \
#        -storepass <password> -keypass <password>
#
#   2. Create android/app/key.properties (do NOT commit this):
#      storePassword=<password>
#      keyPassword=<password>
#      keyAlias=eel-key
#      storeFile=keystore.jks
#
#   3. Run with signing flags:
#      scripts/mobile/build-all-apks.sh --release \
#        --keystore=keystore.jks \
#        --keystore-pass=<password> \
#        --key-alias=eel-key --sign
#
# =============================================================================
# CONTINUOUS INTEGRATION
# =============================================================================
#
# The script is CI-safe. Recommended Woodpecker pipeline step:
#
#   step:
#     name: build-apks
#     image: ghcr.io/cirrusci/flutter:3.24
#     commands:
#       - flutter pub global activate pub_api_client || true
#       - scripts/mobile/build-all-apks.sh \
#           --release --analyze --test \
#           --skip=bvr-kit,bvi-kit \
#           --parallel
#     when:
#       - event: push
#         branch: main
#       - event: tag
#
# Note: bvr-kit and bvi-kit are skipped in CI because the FAP60 and AntIris USB
# devices are not attached to the CI runner. They are built locally before
# hardware arrives.
#
# =============================================================================
# TROUBLESHOOTING
# =============================================================================
#
# Q: citizen-verify fails with "isar_flutter_libs not found".
# A: Use `isar: ^3.1.0` (not `isar_flutter_libs`). Fix: sed -i 's/isar_flutter_libs/isar/g'
#    services/field-mobile-svc/apps/citizen-verify/pubspec.yaml
#
# Q: Dependency version conflicts (geolocator, connectivity_plus).
# A: Let `flutter pub upgrade --major-versions` resolve them automatically, then
#    commit the updated pubspec.lock.
#
# Q: FAP60 / AntIris not detected — biometric capture returns invalid.
# A: Expected behaviour without hardware. Install the FAP60 driver (libFUFinger.so)
#    or AntIris driver (lib_Antiris.so) and connect the USB device. See
#    docs/mobile/M3_HARDWARE_INTEGRATION.md for SDK method signatures.
#
# Q: "No APK found" after a seemingly successful build.
# A: Check build/apks/logs/<app>.log for the actual build command output.
#    Flutter sometimes writes to build/app/outputs/apk/ instead of flutter-apk/.
#
# Q: Test failures — can I still build?
# A: Yes. Omit --test. Tests are a quality gate, not a build requirement.
#    Use `flutter test` inside the specific app directory to investigate.
#
# Q: How do I install the APK on a device?
#   adb install -r build/apks/<app>-debug.apk
#
# Q: How do I install on multiple devices?
#   for f in build/apks/*-debug.apk; do
#     adb install -r "$f"
#   done
#
# Q: How do I uninstall before reinstalling?
#   adb uninstall com.eel.<appname>
#
# =============================================================================

set -euo pipefail

# ---------- argument parsing ----------

MODE="debug"
ONLY=""
SKIP=""
DO_ANALYZE=0
DO_TEST=0
DO_CLEAN=0
DO_PUB_GET=1
PARALLEL=1
KEYSTORE_PATH=""
KEYSTORE_PASS=""
KEY_ALIAS=""

show_help() {
  sed -n '3,209p' "$0" | sed 's/^# //' | head -120
}

for arg in "$@"; do
  case "$arg" in
    --release) MODE="release" ;;
    --debug) MODE="debug" ;;
    --analyze) DO_ANALYZE=1 ;;
    --test) DO_TEST=1 ;;
    --clean) DO_CLEAN=1 ;;
    --no-pub-get) DO_PUB_GET=0 ;;
    --parallel) PARALLEL=2 ;;
    --no-parallel) PARALLEL=1 ;;
    --app=*) ONLY="${arg#--app=}" ;;
    --skip=*) SKIP="${arg#--skip=}" ;;
    --keystore=*) KEYSTORE_PATH="${arg#--keystore=}" ;;
    --keystore-pass=*) KEYSTORE_PASS="${arg#--keystore-pass=}" ;;
    --key-alias=*) KEY_ALIAS="${arg#--key-alias=}" ;;
    --sign)
      if [[ -z "${KEYSTORE_PATH}" ]]; then
        echo "ERROR: --sign requires --keystore=, --keystore-pass=, and --key-alias=" >&2
        exit 2
      fi
      ;;
    -h|--help) show_help; exit 0 ;;
    *) echo "ERROR: unknown argument: $arg" >&2; echo "Run with --help for usage." >&2; exit 2 ;;
  esac
done

# ---------- paths ----------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/build/apks"
LOG_DIR="${OUT_DIR}/logs"
mkdir -p "${OUT_DIR}" "${LOG_DIR}"

if ! command -v flutter >/dev/null 2>&1; then
  echo "ERROR: flutter is not on PATH. Install Flutter 3.19+ and re-run." >&2
  exit 1
fi

# ---------- app registry (plan.md §34) ----------

APPS=(
  "field-mobile-svc/apps/poll-clerk"
  "field-mobile-svc/apps/observer"
  "field-mobile-svc/apps/party-agent"
  "field-mobile-svc/apps/field-support"
  "field-mobile-svc/apps/personnel"
  "field-mobile-svc/apps/citizen-verify"
  "biometric-edge-svc/apps/bvr-kit"
  "biometric-edge-svc/apps/bvi-kit"
  "biometric-edge-svc/apps/diaspora-kit"
  "biometric-edge-svc/apps/legacy-android"
  "rts-svc/apps/rts-kit"
  "training-svc/apps/lms-offline"
  "training-svc/apps/accred-scanner"
  "hello-svc/apps/hello-flutter"
)

is_skipped() { local n="$1"; [[ -n "$SKIP" ]] && [[ ",$SKIP," == *",$n,"* ]]; }
should_build() { local n="$1"; [[ -z "$ONLY" || "$ONLY" == "$n" ]]; }

# ---------- logging helpers ----------

LOG_FILE="${LOG_DIR}/build.log"
: > "${LOG_FILE}"

log()      { tee -a "${LOG_FILE}" > /dev/null; }
section()  { printf '\n====== %s ======\n' "$*" | tee -a "${LOG_FILE}" | cat; }
banner()   { printf '\n%s\n' "$*" | tee -a "${LOG_FILE}" | cat; }

# ---------- summary ----------

declare -a BUILT=() FAILED=() SKIPPED=() ANALYZE_FAIL=() TEST_FAIL=()

# ---------- pre-flight ----------

banner "build-all-apks.sh — started $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
log "mode=${MODE}  only=${ONLY:-all}  skip=${SKIP:-none}"
log "analyze=${DO_ANALYZE}  test=${DO_TEST}  clean=${DO_CLEAN}  parallel=${PARALLEL}"
log "keystore=${KEYSTORE_PATH:-none}  key_alias=${KEY_ALIAS:-none}"
log "out_dir=${OUT_DIR}"

if [[ "${DO_PUB_GET}" -eq 1 ]]; then
  section "workspace: flutter pub get"
  (cd "${ROOT_DIR}" && flutter pub get 2>&1) | log || {
    echo "WARN: workspace pub get failed — per-app retry will follow" >&2
  }
fi

# ---------- per-app build ----------

build_one() {
  local app_path="$1"
  local app_name
  app_name="$(basename "$app_path")"
  local full_path="${ROOT_DIR}/services/${app_path}"
  local log_file="${LOG_DIR}/${app_name}.log"

  if ! should_build "${app_name}"; then return 0; fi
  if is_skipped "${app_name}"; then
    echo "  - skip  ${app_name} (hardware-gated or user-requested)"
    SKIPPED+=("${app_name}")
    return 0
  fi

  if [[ ! -d "${full_path}" ]]; then
    echo "  ✗ missing  ${app_path}" >&2
    FAILED+=("${app_name} (missing)")
    return 0
  fi

  if [[ ! -f "${full_path}/pubspec.yaml" ]]; then
    echo "  - skip   ${app_name}: no pubspec.yaml (not a Flutter app)"
    SKIPPED+=("${app_name}")
    return 0
  fi

  section "app: ${app_name}"
  log "path: ${full_path}"

  # Clean if requested.
  if [[ "${DO_CLEAN}" -eq 1 ]]; then
    echo "  - clean  ${app_name}"
    (cd "${full_path}" && flutter clean 2>&1) | log || true
  fi

  # Dependency resolution.
  echo "  - pubget ${app_name}"
  (cd "${full_path}" && flutter pub get 2>&1) | log || {
    echo "  ✗ pubget ${app_name}: dependencies could not be resolved" >&2
    FAILED+=("${app_name} (pub get)")
    return 0
  }

  # Analyze gate.
  if [[ "${DO_ANALYZE}" -eq 1 ]]; then
    echo "  - analyse ${app_name}"
    if ! (cd "${full_path}" && flutter analyze 2>&1) | tee -a "${log_file}" | log; then
      echo "  ✗ analyse ${app_name}: see logs"
      ANALYZE_FAIL+=("${app_name}")
    fi
  fi

  # Test gate.
  if [[ "${DO_TEST}" -eq 1 ]]; then
    echo "  - test   ${app_name}"
    if ! (cd "${full_path}" && flutter test 2>&1) | tee -a "${log_file}" | log; then
      echo "  ✗ test   ${app_name}: see logs"
      TEST_FAIL+=("${app_name}")
    fi
  fi

  # APK build.
  echo "  - build  ${app_name} (${MODE})"
  local build_cmd=(flutter build apk --"${MODE}")

  if [[ -n "${KEYSTORE_PATH}" && -n "${KEYSTORE_PASS}" && -n "${KEY_ALIAS}" ]]; then
    build_cmd+=(
      --dart-define=KEYSTORE_PATH="${KEYSTORE_PATH}"
      --dart-define=KEYSTORE_PASS="${KEYSTORE_PASS}"
      --dart-define=KEY_ALIAS="${KEY_ALIAS}"
    )
  fi

  if ! (cd "${full_path}" && "${build_cmd[@]}" 2>&1) | tee -a "${log_file}" | log; then
    echo "  x build  ${app_name}: build command failed" >&2
    FAILED+=("${app_name}")
    return 0
  fi

  # Locate and copy APK.
  local apk_src
  apk_src="$(find "${full_path}/build/app/outputs" \
    -name "app-${MODE}.apk" -type f 2>/dev/null | head -1 || true)"

  if [[ -n "${apk_src}" && -f "${apk_src}" ]]; then
    cp "${apk_src}" "${OUT_DIR}/${app_name}-${MODE}.apk"
    local size
    size="$(du -h "${OUT_DIR}/${app_name}-${MODE}.apk" | cut -f1)"
    echo "    ok  ${app_name}-${MODE}.apk  (${size})"
    BUILT+=("${app_name}")
  else
    echo "  x ${app_name}: build succeeded but APK not found" >&2
    echo "    (Flutter may have written to a different output directory)" >&2
    FAILED+=("${app_name} (no apk)")
  fi
}

export -f build_one log section banner
export ROOT_DIR OUT_DIR LOG_DIR MODE ONLY SKIP DO_ANALYZE DO_TEST DO_CLEAN
export KEYSTORE_PATH KEYSTORE_PASS KEY_ALIAS
export -f is_skipped should_build

# ---------- run (parallel or serial) ----------

section "building ${#APPS[@]} apps"

if [[ "${PARALLEL}" -gt 1 ]] && command -v xargs >/dev/null 2>&1; then
  printf '%s\n' "${APPS[@]}" | xargs -n1 -P"${PARALLEL}" -I{} \
    bash -c 'build_one "$@"' _ {}
else
  for app in "${APPS[@]}"; do build_one "${app}"; done
fi

# ---------- summary ----------

banner "summary"

{
  # Compact JSON summary (safe even when arrays are empty).
  printf '{'
  printf '"built":%s,' "$(printf '%s\n' "${BUILT[@]:-}" | \
    jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"failed":%s,' "$(printf '%s\n' "${FAILED[@]:-}" | \
    jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"skipped":%s,' "$(printf '%s\n' "${SKIPPED[@]:-}" | \
    jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"analyze_fail":%s,' "$(printf '%s\n' "${ANALYZE_FAIL[@]:-}" | \
    jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"test_fail":%s,' "$(printf '%s\n' "${TEST_FAIL[@]:-}" | \
    jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"mode":"%s",' "${MODE}"
  printf '"timestamp":"%s"' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  printf '}\n'
} > "${OUT_DIR}/summary.json"

printf '\n'
printf '=%.0s' {1..55}
printf '\n'
printf '  Built         %d  %s\n' "${#BUILT[@]}" "${BUILT[*]:-—}"
printf '  Failed        %d  %s\n' "${#FAILED[@]}" "${FAILED[*]:-—}"
printf '  Skipped       %d  %s\n' "${#SKIPPED[@]}" "${SKIPPED[*]:-—}"
if [[ "${DO_ANALYZE}" -eq 1 ]]; then
  printf '  Analyze fail  %d  %s\n' "${#ANALYZE_FAIL[@]}" "${ANALYZE_FAIL[*]:-—}"
fi
if [[ "${DO_TEST}" -eq 1 ]]; then
  printf '  Test fail     %d  %s\n' "${#TEST_FAIL[@]}" "${TEST_FAIL[*]:-—}"
fi
printf '=%.0s' {1..55}
printf '\n\n'
printf '  APKs  %s/\n' "${OUT_DIR}"
ls -lh "${OUT_DIR}"/*.apk 2>/dev/null || echo '  (none produced)'
printf '\n'
printf '  Logs  %s/\n' "${LOG_DIR}"
printf '  JSON  %s/summary.json\n' "${OUT_DIR}"
printf '  =%.0s\n' {1..55}
printf '\n'

# Exit non-zero if anything meaningful failed.
if [[ ${#FAILED[@]} -gt 0 || ${#ANALYZE_FAIL[@]} -gt 0 || ${#TEST_FAIL[@]} -gt 0 ]]; then
  exit 1
fi
