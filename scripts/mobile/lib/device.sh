#!/usr/bin/env bash
# device.sh — Shared adb/device helpers for factory drill scripts.
# Sourced after lib/common.sh. Bash 3.2 compatible (macOS system bash).
# Not executable directly.
#
# NOTE on set -e + pipefail: every helper below is typically called inside
# $(...) assignments. A failing pipeline there would kill the caller
# silently, so each helper neutralises EXPECTED non-zero statuses with
# `|| true` and lets callers decide via empty output. Genuine transport
# failures surface as empty results; eel_require_device() upfront plus the
# callers' explicit emptiness checks turn those into clear errors.
# Rule: no helper may propagate a non-zero status for an empty result.

# --- prerequisites -----------------------------------------------------------

# Fail unless adb exists and DEVICE is visible via `adb devices`.
eel_require_device() {
  local device="$1"
  command -v adb >/dev/null 2>&1 || { _log_error "adb not found in PATH (install Android SDK Platform Tools)"; return 1; }
  if ! adb devices | tr -d '\r' | awk 'NR>1 && $2=="device" {print $1}' | grep -qx "$device"; then
    _log_error "device '$device' not online (see: adb devices)"
    return 1
  fi
}

# --- package / component resolution ------------------------------------------

# Print the single installed package whose name contains FRAGMENT.
# Fails when zero or more than one match (ambiguous).
eel_resolve_pkg() {
  local device="$1" fragment="$2" matches pkg count alt
  # App slugs use hyphens (bvi-kit) while package names use underscores
  # (com.eel.bvi_kit) — match either spelling.
  alt=$(printf '%s' "$fragment" | tr '-' '_')
  # grep finds nothing on zero matches (exit 1) — neutralised so the
  # explicit count checks below produce the clear error.
  matches=$(adb -s "$device" shell pm list packages 2>/dev/null | tr -d '\r' | grep -F -e "$fragment" -e "$alt" | cut -d: -f2 | sort -u || true)
  count=$(printf '%s' "$matches" | grep -c .)
  if [[ "$count" -eq 0 ]]; then
    _log_error "no installed package matches '$fragment' on $device"
    return 1
  fi
  if [[ "$count" -gt 1 ]]; then
    _log_error "ambiguous fragment '$fragment' matches: $(echo "$matches" | tr '\n' ' ')"
    return 1
  fi
  pkg=$(printf '%s' "$matches" | head -1)
  printf '%s' "$pkg"
}

# Print the launcher component (pkg/Activity) for PKG, or empty string when
# it cannot be resolved. Never fails — callers fall back to monkey.
eel_resolve_launcher() {
  local device="$1" pkg="$2" component
  component=$(adb -s "$device" shell cmd package resolve-activity --brief "$pkg" 2>/dev/null \
    | tr -d '\r' | tail -1 || true)
  case "$component" in
    */*) printf '%s' "$component" ;;
    *) printf '' ;;
  esac
}

# --- process -----------------------------------------------------------------

eel_pid() {
  local device="$1" pkg="$2"
  # pidof exits 1 when no process matches — an empty result, not an error.
  adb -s "$device" shell pidof "$pkg" 2>/dev/null | tr -d '\r' | head -1 || true
}

# --- logcat crash detection --------------------------------------------------

eel_logcat_clear() {
  local device="$1"
  adb -s "$device" logcat -c 2>/dev/null || true
}

# Dump logcat to FILE and return 0 when a FATAL EXCEPTION for PKG is found.
eel_logcat_crashcheck() {
  local device="$1" pkg="$2" file="$3"
  adb -s "$device" logcat -d > "$file" 2>/dev/null || true
  if grep -q "FATAL EXCEPTION" "$file" 2>/dev/null; then
    if grep -A3 "FATAL EXCEPTION" "$file" | grep -qF "$pkg"; then
      return 0
    fi
    # Unscoped fatal still counts when the crashing process can't be
    # attributed (conservative: a crash during the drill fails the drill).
    return 0
  fi
  return 1
}

# --- launch ------------------------------------------------------------------

# Cold-launch PKG: force-stop first (clears stale tasks/permission dialogs
# that would otherwise swallow the launch), then start. Prints am start
# TotalTime ms on success (empty otherwise).
# Uses the resolved launcher when available, monkey as fallback.
eel_launch() {
  local device="$1" pkg="$2" component out ms
  adb -s "$device" shell am force-stop "$pkg" >/dev/null 2>&1 || true
  sleep 1
  component=$(eel_resolve_launcher "$device" "$pkg")
  if [[ -n "$component" ]]; then
    out=$(adb -s "$device" shell am start -W -n "$component" 2>&1 | tr -d '\r' || true)
    # No TotalTime line (e.g. warm delivery) is an empty result, not an error.
    ms=$(printf '%s' "$out" | grep -E '^TotalTime:' | awk '{print $2}' || true)
    printf '%s' "${ms:-}"
    return 0
  fi
  _log_warn "launcher not resolvable for $pkg; falling back to monkey"
  adb -s "$device" shell monkey -p "$pkg" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1
  printf ''
  return 0
}

# --- airplane mode -----------------------------------------------------------

# Set airplane mode on|off and verify the setting took effect. Retries once.
eel_airplane() {
  local device="$1" want="$2" val expect
  if [[ "$want" == "on" ]]; then expect=1; else expect=0; fi
  for _attempt in 1 2; do
    adb -s "$device" shell settings put global airplane_mode_on "$expect" >/dev/null 2>&1
    if [[ "$expect" == "1" ]]; then
      adb -s "$device" shell am broadcast -a android.intent.action.AIRPLANE_MODE --ez state true >/dev/null 2>&1
    else
      adb -s "$device" shell am broadcast -a android.intent.action.AIRPLANE_MODE --ez state false >/dev/null 2>&1
    fi
    sleep 2
    val=$(adb -s "$device" shell settings get global airplane_mode_on 2>/dev/null | tr -d '\r[:space:]' || true)
    if [[ "$val" == "$expect" ]]; then
      return 0
    fi
  done
  _log_error "airplane mode did not switch $want (still reports '$val')"
  return 1
}
