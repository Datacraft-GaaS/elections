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

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PASS=0
FAIL=0

run_check() {
  local lang="$1"
  local script="${SCRIPT_DIR}/${lang}/generate.sh"
  if [[ ! -f "$script" ]]; then
    echo "FAIL [${lang}]: generate.sh not found at ${script}"
    FAIL=$((FAIL + 1))
    return
  fi
  if bash "$script" --check; then
    echo "PASS [${lang}]"
    PASS=$((PASS + 1))
  else
    echo "FAIL [${lang}]: generate.sh --check exited non-zero"
    FAIL=$((FAIL + 1))
  fi
}

run_check dart
run_check ts
run_check py
run_check rs

echo ""
echo "Results: ${PASS} passed, ${FAIL} failed"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
exit 0
