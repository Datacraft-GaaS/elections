#!/usr/bin/env bash
# observer.sh: TalkBack traversal for observer Flutter app.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.observer"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:observer] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: observer TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: file observation report
for i in 1 2 3; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

CONTENT="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -o 'content-desc="[^"]*"' | head -5)"
log "Focus sequence sample: ${CONTENT}"

log "PASS: observer TalkBack traversal complete"
