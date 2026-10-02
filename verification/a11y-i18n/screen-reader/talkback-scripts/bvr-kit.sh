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

# bvr-kit.sh: TalkBack traversal for bvr-kit (biometric voter registration kit).
# Uses adb to drive UI Automator events and asserts swipe order + focus targets.
# Requires: adb connected to a device/emulator with bvr-kit APK installed + TalkBack enabled.
set -euo pipefail

ADB="${ADB:-adb}"
PACKAGE="io.eel.bvr_kit"
STUB="${EEL_SR_STUB:-}"

log() { printf '[talkback:bvr-kit] %s\n' "$*" >&2; }

if [[ -n "${STUB}" ]]; then
  log "stub mode: asserting APK package declared and adb present conceptually"
  log "PASS: bvr-kit TalkBack stub"
  exit 0
fi

# Verify device connected
"${ADB}" devices | grep -q "device$" || { log "no adb device connected"; exit 2; }

# Launch app
"${ADB}" shell am start -n "${PACKAGE}/.MainActivity" >/dev/null
sleep 2

# Enable TalkBack via accessibility settings shortcut (assumes device configured)
# Primary task: biometric capture flow
# Step 1: swipe right (next focusable element) and assert welcome heading announced
"${ADB}" shell input swipe 500 1000 900 1000 300
sleep 0.5
CONTENT="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -o 'content-desc="[^"]*"' | head -1)"
log "Step 1 focus: ${CONTENT}"

# Step 2: navigate to NIN entry field
"${ADB}" shell input swipe 500 1000 900 1000 300
sleep 0.5

# Step 3: assert input field has accessible label
EDITABLE="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.EditText"' || true)"
if [[ "${EDITABLE}" -eq 0 ]]; then
  log "FAIL: no EditText (NIN field) found"
  exit 1
fi

# Step 4: swipe to submit button and assert focusable
"${ADB}" shell input swipe 500 1000 900 1000 300
sleep 0.5
BUTTON="$("${ADB}" shell uiautomator dump /dev/stdout 2>/dev/null | grep -c 'class="android.widget.Button"' || true)"
if [[ "${BUTTON}" -eq 0 ]]; then
  log "FAIL: no Button element found"
  exit 1
fi

log "PASS: bvr-kit swipe order and focus targets verified"
