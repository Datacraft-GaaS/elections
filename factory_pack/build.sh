#!/usr/bin/env bash
# build.sh — Reproducible release build for a Flutter app.
# Usage: scripts/mobile/build.sh <APP> --flavor dev|staging|production [--target-platform android]
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
APP="${1:-}"; shift || true
FLAVOR=""; TARGET_PLATFORM="android"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --flavor) FLAVOR="$2"; shift 2 ;;
    --target-platform) TARGET_PLATFORM="$2"; shift 2 ;;
    *) _log_error "Unknown flag: $1"; exit 1 ;;
  esac
done
[[ -z "$APP" || -z "$FLAVOR" ]] && { _log_error "Usage: scripts/mobile/build.sh <APP> --flavor dev|staging|production"; exit 1; }
APP_PATH=$(get_app_path "$APP")
[[ -z "$APP_PATH" ]] && { _log_error "Unknown app: $APP"; exit 1; }
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${REPO_ROOT}/dist/mobile/${APP}"
mkdir -p "$BUILD_DIR"
BUILD_EPOCH=$(git log -1 --format=%ct 2>/dev/null || echo "0")
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
_log_info "Building $APP (flavor=$FLAVOR, platform=$TARGET_PLATFORM)..."
case "$TARGET_PLATFORM" in
  android)
    # --split-debug-info emits symbol maps so production stack traces get
    # file/line info (no --obfuscate: symbols stay readable). Archived next
    # to the SBOM in dist (symbols/<app>/).
    flutter build apk --flavor "$FLAVOR" \
      --dart-define=EEL_BUILD_EPOCH="$BUILD_EPOCH" \
      --dart-define=EEL_BUILD_SHA="$GIT_SHA" \
      --dart-define=EEL_FLAVOR="$FLAVOR" \
      --split-debug-info="$BUILD_DIR/symbols" \
      -t "$APP_PATH/lib/main.dart"
    cp "$APP_PATH/build/app/outputs/flutter-apk/app-${FLAVOR}-release.apk" "$BUILD_DIR/${APP}-${FLAVOR}-release.apk" 2>/dev/null || true
    ARTIFACT="$BUILD_DIR/${APP}-${FLAVOR}-release.apk"
    SIZE_MB=0
    if [[ -f "$ARTIFACT" ]]; then
      SIZE_BYTES=$(stat -f%z "$ARTIFACT" 2>/dev/null || stat -c%s "$ARTIFACT" 2>/dev/null || echo 0)
      SIZE_MB=$(( (SIZE_BYTES + 1048575) / 1048576 ))
      # Plan §7/§9 budgets: ≤ 40 MB per app, ≤ 60 MB for kits (native libs).
      case "$APP" in
        *kit*) BUDGET_MB=60 ;;
        *) BUDGET_MB=40 ;;
      esac
      if [[ "${EEL_SIZE_BUDGET_MB:-}" =~ ^[0-9]+$ ]]; then BUDGET_MB="$EEL_SIZE_BUDGET_MB"; fi
      _log_info "Artifact size: ${SIZE_MB} MB (budget ${BUDGET_MB} MB)"
      if [[ "$SIZE_MB" -gt "$BUDGET_MB" ]]; then
        _log_error "SIZE GATE FAILED: $ARTIFACT is ${SIZE_MB} MB, budget ${BUDGET_MB} MB (override: EEL_SIZE_BUDGET_MB)"
        exit 1
      fi
    else
      _log_warn "Release APK not found at $ARTIFACT — size gate skipped"
    fi
    ;;
  ios)
    flutter build ios --flavor "$FLAVOR" \
      --dart-define=EEL_BUILD_EPOCH="$BUILD_EPOCH" \
      --dart-define=EEL_BUILD_SHA="$GIT_SHA" \
      --dart-define=EEL_FLAVOR="$FLAVOR" \
      --release --no-codesign -t "$APP_PATH/lib/main.dart"
    ;;
  *) _log_error "Unsupported target platform: $TARGET_PLATFORM"; exit 1 ;;
esac
cat > "$BUILD_DIR/manifest.json" <<EOF
{"app":"$APP","flavor":"$FLAVOR","platform":"$TARGET_PLATFORM","git_sha":"$GIT_SHA","build_epoch":"$BUILD_EPOCH","sbom_path":"sbom/spdx.json","artifact_path":"${APP}-${FLAVOR}-release.apk","size_mb":${SIZE_MB:-0}}
EOF
_log_info "Build complete: $BUILD_DIR"
