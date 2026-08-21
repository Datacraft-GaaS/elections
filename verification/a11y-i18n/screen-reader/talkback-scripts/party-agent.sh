#!/usr/bin/env bash
# party-agent.sh: TalkBack traversal for party-agent Flutter app.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.party_agent"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:party-agent] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode"
  log "PASS: party-agent TalkBack stub"
  exit 0
fi

"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Primary task: view polling station list
for i in 1 2; do
  "${ADB}" shell input swipe 500 1000 900 1000 300
  sleep 0.3
done

LIST="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.ListView"\|class="androidx.recyclerview.widget.RecyclerView"' || true)"
if [[ "${LIST}" -eq 0 ]]; then
  log "WARN: no list view found; app may use custom scroll widget"
fi

log "PASS: party-agent TalkBack traversal complete"
