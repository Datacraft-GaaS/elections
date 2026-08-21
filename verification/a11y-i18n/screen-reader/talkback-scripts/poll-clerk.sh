#!/usr/bin/env bash
# poll-clerk.sh: TalkBack traversal for poll-clerk Flutter app.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.poll_clerk"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:poll-clerk] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: poll-clerk TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: voter check-in
for i in 1 2 3; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

EDITABLE="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.EditText"' || true)"
if [[ "${EDITABLE}" -eq 0 ]]; then
  log "FAIL: no voter ID input field"
  exit 1
fi

log "PASS: poll-clerk TalkBack traversal complete"
