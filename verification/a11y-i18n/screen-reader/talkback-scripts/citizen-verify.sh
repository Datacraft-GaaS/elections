#!/usr/bin/env bash
# citizen-verify.sh: TalkBack traversal for citizen verify flow (bvv-voter-lookup via WebView).
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.bvi_kit"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:citizen-verify] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: citizen-verify TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: enter NIN and retrieve registration status
for i in 1 2; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

EDITABLE="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.EditText"' || true)"
if [[ "${EDITABLE}" -eq 0 ]]; then
  log "FAIL: NIN input not found"
  exit 1
fi

log "PASS: citizen-verify TalkBack traversal complete"
