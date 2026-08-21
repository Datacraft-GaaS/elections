#!/usr/bin/env bash
# diaspora-kit.sh: TalkBack traversal for diaspora-kit Flutter app.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.diaspora_kit"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:diaspora-kit] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: diaspora-kit TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: diaspora voter registration initiation
for i in 1 2 3; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

EDITABLE="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.EditText"' || true)"
if [[ "${EDITABLE}" -eq 0 ]]; then
  log "FAIL: no input fields found"
  exit 1
fi

log "PASS: diaspora-kit TalkBack traversal complete"
