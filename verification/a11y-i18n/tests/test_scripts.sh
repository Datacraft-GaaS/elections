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

# test_scripts.sh: Smoke tests for verification/a11y-i18n.
# Checks: bash -n clean on all .sh, python -m py_compile on all .py,
#         YAML validity, and that every UI in ui-inventory.yaml maps to a real directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
A11Y_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${A11Y_DIR}/../.." && pwd)"
INVENTORY="${A11Y_DIR}/ui-inventory.yaml"

PASS=0
FAIL=0

ok() { printf '[PASS] %s\n' "$*"; PASS=$((PASS + 1)); }
fail() { printf '[FAIL] %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }

# --- bash -n on all .sh files ---
while IFS= read -r -d '' f; do
  if bash -n "${f}" 2>/dev/null; then
    ok "bash -n: ${f#"${REPO_ROOT}/"}"
  else
    fail "bash -n: ${f#"${REPO_ROOT}/"}"
  fi
done < <(find "${A11Y_DIR}" -name "*.sh" -print0)

# --- shellcheck (optional) ---
if command -v shellcheck >/dev/null 2>&1; then
  while IFS= read -r -d '' f; do
    if shellcheck --shell=bash --severity=error "${f}" 2>/dev/null; then
      ok "shellcheck: ${f#"${REPO_ROOT}/"}"
    else
      fail "shellcheck: ${f#"${REPO_ROOT}/"}"
    fi
  done < <(find "${A11Y_DIR}" -name "*.sh" -print0)
else
  echo "[SKIP] shellcheck not installed"
fi

# --- python -m py_compile on all .py ---
while IFS= read -r -d '' f; do
  if python3 -m py_compile "${f}" 2>/dev/null; then
    ok "py_compile: ${f#"${REPO_ROOT}/"}"
  else
    fail "py_compile: ${f#"${REPO_ROOT}/"}"
  fi
done < <(find "${A11Y_DIR}" -name "*.py" -print0)

# --- YAML validity ---
while IFS= read -r -d '' f; do
  if python3 -c "import yaml; yaml.safe_load(open('${f}'))" 2>/dev/null; then
    ok "yaml: ${f#"${REPO_ROOT}/"}"
  else
    fail "yaml: ${f#"${REPO_ROOT}/"}"
  fi
done < <(find "${A11Y_DIR}" -name "*.yaml" -print0)

# --- Every UI in ui-inventory.yaml corresponds to a real directory ---
python3 - <<PYEOF
import yaml, os, sys

inventory = yaml.safe_load(open('${INVENTORY}'))
repo_root = '${REPO_ROOT}'
failures = []
passes = 0

for s in inventory['surfaces']:
    d = os.path.join(repo_root, s['dir'])
    if os.path.isdir(d):
        print(f'[PASS] dir exists: {s["dir"]}')
        passes += 1
    else:
        print(f'[FAIL] dir missing: {s["dir"]} (surface: {s["name"]})', file=sys.stderr)
        failures.append(s['name'])

print(f'Directory check: {passes} pass, {len(failures)} fail')
if failures:
    sys.exit(1)
PYEOF

if [[ "${FAIL}" -eq 0 ]]; then
  echo ""
  echo "All checks passed (${PASS} pass, 0 fail)"
  exit 0
else
  echo ""
  echo "FAIL: ${FAIL} check(s) failed, ${PASS} passed" >&2
  exit 1
fi
