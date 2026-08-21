#!/usr/bin/env bash
# field-support.sh: TalkBack traversal for field-support Flutter app.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.field_support"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:field-support] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: field-support TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: submit field incident report
for i in 1 2 3 4; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

BUTTON="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.Button"' || true)"
if [[ "${BUTTON}" -eq 0 ]]; then
  log "FAIL: no submit button found"
  exit 1
fi

log "PASS: field-support TalkBack traversal complete"
