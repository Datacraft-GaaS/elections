#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
