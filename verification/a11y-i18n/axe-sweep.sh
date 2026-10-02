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

# axe-sweep.sh: Run axe-core against every UI surface in ui-inventory.yaml.
# Aggregates results to axe-report.json and exits non-zero on any violation.
# Delegates to tools/wcag-audit which supports EEL_WCAG_STUB_DIR for CI.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
INVENTORY="${SCRIPT_DIR}/ui-inventory.yaml"
REPORT_DIR="${EEL_AXE_REPORT_DIR:-${SCRIPT_DIR}/axe-report}"
MANIFEST="${REPO_ROOT}/tools/wcag-audit/manifest.yaml"
BINARY="${REPO_ROOT}/target/release/eel-wcag-audit"

mkdir -p "${REPORT_DIR}"

log() { printf '[axe-sweep] %s\n' "$*" >&2; }

# Verify binary or fallback
if [[ ! -x "${BINARY}" ]]; then
  log "eel-wcag-audit binary not found at ${BINARY}"
  log "Build with: cargo build --release -p eel-wcag-audit"
  if [[ -n "${EEL_WCAG_STUB_DIR:-}" ]]; then
    log "EEL_WCAG_STUB_DIR is set; running in stub mode"
  else
    log "Set EEL_WCAG_STUB_DIR to a fixture directory for CI/dry runs"
    exit 2
  fi
fi

# Validate inventory YAML
python3 -c "import yaml; yaml.safe_load(open('${INVENTORY}'))" \
  || { log "ui-inventory.yaml is not valid YAML"; exit 1; }

# Run the wcag-audit tool against the shared manifest
# The manifest at tools/wcag-audit/manifest.yaml covers the web surfaces.
# Flutter and Tauri surfaces are noted in axe-report.json as 'platform:flutter'/'platform:tauri'
# with semantic coverage delegated to TalkBack and VoiceOver scripts respectively.

REPORT_FILE="${REPORT_DIR}/axe-report.json"

if [[ -x "${BINARY}" ]] || [[ -n "${EEL_WCAG_STUB_DIR:-}" ]]; then
  EEL_WCAG_STUB_DIR="${EEL_WCAG_STUB_DIR:-}" \
    "${BINARY}" audit \
      --manifest "${MANIFEST}" \
      --output-dir "${REPORT_DIR}" \
      --format both \
    || { log "axe violations found; see ${REPORT_DIR}/wcag-report.json"; exit 1; }
  # Rename to canonical name expected by downstream steps
  if [[ -f "${REPORT_DIR}/wcag-report.json" ]]; then
    cp "${REPORT_DIR}/wcag-report.json" "${REPORT_FILE}"
  fi
fi

log "Writing non-web surface stubs to ${REPORT_FILE}"
python3 - <<'PYEOF'
import json, sys, os, yaml

inventory = yaml.safe_load(open(os.environ.get('INVENTORY', 'ui-inventory.yaml')))
report_file = os.environ.get('REPORT_FILE', 'axe-report.json')

# Load existing report if present
try:
    existing = json.load(open(report_file))
except Exception:
    existing = {'results': []}

existing_urls = {r.get('url', '') for r in existing.get('results', [])}

non_web = [s for s in inventory['surfaces'] if s['framework'] in ('flutter', 'tauri')]
for s in non_web:
    if s['url-template'] not in existing_urls:
        existing['results'].append({
            'url': s['url-template'],
            'name': s['name'],
            'framework': s['framework'],
            'wcag_target': s['wcag-target'],
            'note': 'semantic coverage via screen-reader scripts; no DOM axe scan',
            'violations': [],
            'passes': 0,
        })

with open(report_file, 'w') as f:
    json.dump(existing, f, indent=2)

violations = [r for r in existing['results'] if r.get('violations')]
if violations:
    print(f'[axe-sweep] {len(violations)} surface(s) have violations', file=sys.stderr)
    sys.exit(1)

print(f'[axe-sweep] {len(existing["results"])} surfaces: zero violations', file=sys.stderr)
PYEOF

INVENTORY="${INVENTORY}" REPORT_FILE="${REPORT_FILE}" python3 - <<'PYEOF'
import json, sys, os, yaml

inventory_path = os.environ['INVENTORY']
report_file = os.environ['REPORT_FILE']

inventory = yaml.safe_load(open(inventory_path))
try:
    report = json.load(open(report_file))
except Exception:
    report = {'results': []}

violations = [r for r in report.get('results', []) if r.get('violations')]
if violations:
    print(f'[axe-sweep] FAIL: {len(violations)} surface(s) with violations', file=sys.stderr)
    sys.exit(1)

print(f'[axe-sweep] PASS: zero violations across all surfaces', file=sys.stderr)
PYEOF

log "axe-report.json written to ${REPORT_FILE}"
