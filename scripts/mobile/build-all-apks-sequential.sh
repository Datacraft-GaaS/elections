#!/usr/bin/env bash
# scripts/mobile/build-all-apks-sequential.sh
#
# Builds each mobile APK sequentially, one at a time, with Gradle daemon
# killed between apps. Each build gets its own clean JVM (Java 17) to avoid
# the memory contention issue that breaks parallel batch builds.
#
# Usage:
#   scripts/mobile/build-all-apks-sequential.sh --release
#   scripts/mobile/build-all-apks-sequential.sh --app=hello-flutter
#
# Designed for the Java 17 / Gradle 8.14 / AGP 8.3.2 / Kotlin 2.0.21 stack.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

MODE="debug"
ONLY=""
SKIP=""
DO_CLEAN=1
JAVA_HOME="/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home"

for arg in "$@"; do
  case "$arg" in
    --release) MODE="release" ;;
    --debug) MODE="debug" ;;
    --clean) DO_CLEAN=1 ;;
    --no-clean) DO_CLEAN=0 ;;
    --app=*) ONLY="${arg#--app=}" ;;
    --skip=*) SKIP="${arg#--skip=}" ;;
    -h|--help) echo "Usage: $0 [--release] [--clean] [--app=X] [--skip=X,Y]"; exit 0 ;;
  esac
done

export JAVA_HOME
export PATH="$JAVA_HOME/bin:$PATH"

is_skipped() { local n="$1"; [[ -n "$SKIP" ]] && [[ ",$SKIP," == *",$n,"* ]]; }
should_build() { local n="$1"; [[ -z "$ONLY" || "$ONLY" == "$n" ]]; }

APPS=(
  "poll-clerk" "observer" "party-agent" "field-support" "personnel"
  "citizen-verify" "bvr-kit" "bvi-kit" "diaspora-kit"
  "legacy-android" "rts-kit" "lms-offline" "accred-scanner" "hello-flutter"
)

OUT_DIR="${ROOT_DIR}/build/apks"
LOG_DIR="${OUT_DIR}/logs"
mkdir -p "${OUT_DIR}" "${LOG_DIR}"

BUILT=()
FAILED=()
SKIPPED=()
RUN_START=$(date +%s)

echo "=== Sequential APK build (${MODE}) ==="
echo "    Java:  $("$JAVA_HOME/bin/java" -version 2>&1 | head -1)"
echo "    Output: ${OUT_DIR}/"
echo "    Logs:   ${LOG_DIR}/"
echo ""

for app in "${APPS[@]}"; do
  should_build "$app" || continue
  if is_skipped "$app"; then
    echo "skip $app"
    SKIPPED+=("$app")
    continue
  fi

  full_path=""
  case "$app" in
    poll-clerk|observer|party-agent|field-support|personnel|citizen-verify)
      full_path="${ROOT_DIR}/services/field-mobile-svc/apps/$app" ;;
    bvr-kit|bvi-kit|diaspora-kit|legacy-android)
      full_path="${ROOT_DIR}/services/biometric-edge-svc/apps/$app" ;;
    rts-kit) full_path="${ROOT_DIR}/services/rts-svc/apps/$app" ;;
    lms-offline|accred-scanner) full_path="${ROOT_DIR}/services/training-svc/apps/$app" ;;
    hello-flutter) full_path="${ROOT_DIR}/services/hello-svc/apps/$app" ;;
  esac

  if [[ ! -d "${full_path}" ]]; then
    echo "FAIL $app (missing dir)"
    FAILED+=("$app")
    continue
  fi

  echo "==> $app (${MODE})"
  start=$(date +%s)

  # Kill Gradle daemon to free 8GB RAM for next build.
  (cd "${full_path}/android" && ./gradlew --stop 2>/dev/null) || true
  find ~/.gradle/daemon -name 'registry.bin*' -delete 2>/dev/null || true

  if [[ "${DO_CLEAN}" -eq 1 ]]; then
    (cd "${full_path}" && JAVA_HOME="$JAVA_HOME" PATH="$JAVA_HOME/bin:$PATH" flutter clean 2>/dev/null) || true
  fi

  log_file="${LOG_DIR}/${app}.log"
  : > "${log_file}"

  if (cd "${full_path}" && JAVA_HOME="$JAVA_HOME" PATH="$JAVA_HOME/bin:$PATH" \
        flutter build apk --"${MODE}" --android-skip-build-dependency-validation 2>&1) >> "${log_file}" 2>&1; then
    apk_src=$(find "${full_path}/build/app/outputs" -name "app-${MODE}.apk" -type f 2>/dev/null | head -1 || true)
    if [[ -n "${apk_src}" && -f "${apk_src}" ]]; then
      cp "${apk_src}" "${OUT_DIR}/${app}-${MODE}.apk"
      size=$(du -h "${OUT_DIR}/${app}-${MODE}.apk" | cut -f1)
      elapsed=$(( $(date +%s) - start ))
      echo "    ok $app (${size}) in ${elapsed}s"
      BUILT+=("$app")
    else
      echo "    FAIL $app (no APK)"
      FAILED+=("$app")
    fi
  else
    elapsed=$(( $(date +%s) - start ))
    echo "    FAIL $app (${elapsed}s) — see ${log_file}"
    FAILED+=("$app")
  fi
done

elapsed_total=$(( $(date +%s) - RUN_START ))

# JSON summary.
{
  printf '{'
  printf '"built":%s,' "$(printf '%s\n' "${BUILT[@]:-}" | jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"failed":%s,' "$(printf '%s\n' "${FAILED[@]:-}" | jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"skipped":%s,' "$(printf '%s\n' "${SKIPPED[@]:-}" | jq -Rs 'split("\n") | map(select(length > 0))' 2>/dev/null || echo '[]')"
  printf '"mode":"%s",' "${MODE}"
  printf '"timestamp":"%s",' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  printf '"duration_seconds":%d' "${elapsed_total}"
  printf '}\n'
} > "${OUT_DIR}/summary.json"

echo ""
echo "=== Summary ==="
printf 'Built:    %d  %s\n' "${#BUILT[@]}" "${BUILT[*]:-—}"
printf 'Failed:   %d  %s\n' "${#FAILED[@]}" "${FAILED[*]:-—}"
printf 'Skipped:  %d  %s\n' "${#SKIPPED[@]}" "${SKIPPED[*]:-—}"
printf 'Time:     %ds\n' "${elapsed_total}"
echo ""
echo "APKs:"
ls -lh "${OUT_DIR}/"*.apk 2>/dev/null | awk '{printf "  %s  %s\n", $9, $5}' || echo "  (none)"

[[ ${#FAILED[@]} -eq 0 ]] || exit 1
