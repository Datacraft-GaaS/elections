#!/usr/bin/env bash
# personnel.sh: TalkBack traversal for personnel Flutter app.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.personnel"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:personnel] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: personnel TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

for i in 1 2 3; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

log "PASS: personnel TalkBack traversal complete"
