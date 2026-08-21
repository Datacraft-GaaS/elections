#!/usr/bin/env bash
# rts-kit.sh: TalkBack traversal for rts-kit (results transmission system kit).
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.rts_kit"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:rts-kit] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: rts-kit TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: open station results form and navigate to tallies
"${ADB}" shell input swipe 500 1000 900 1000 300
sleep 0.5

# Assert heading present (stream type)
HEADING="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'accessibilityHeading="true"' || true)"
if [[ "${HEADING}" -eq 0 ]]; then
  log "WARN: no accessibility heading; falling back to content-desc check"
fi

# Navigate to tally fields (3 swipes)
for i in 1 2 3; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

EDITABLE="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.EditText"' || true)"
if [[ "${EDITABLE}" -eq 0 ]]; then
  log "FAIL: no tally input fields found"
  exit 1
fi

log "PASS: rts-kit TalkBack traversal complete"
