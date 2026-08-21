#!/usr/bin/env bash
# 9.2-surface-inventory.sh: Assert every Appendix-A surface has a deployable artefact.
# Checks: chart dir, pubspec.yaml (Flutter), tauri.conf.json (Tauri), or astro/ex/django dir.
set -euo pipefail

REPO_ROOT="${EEL_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)}"
AUDIT_MODE="${EEL_AUDIT_MODE:-}"
FIXTURES_DIR="${EEL_FIXTURES_DIR:-${REPO_ROOT}/verification/completeness/fixtures/mini-audit}"
INVENTORY="${REPO_ROOT}/verification/a11y-i18n/ui-inventory.yaml"

log() { printf '[9.2-surface-inventory] %s\n' "$*" >&2; }

if [[ -n "${AUDIT_MODE}" && "${AUDIT_MODE}" == "dry" ]]; then
  log "dry mode: checking fixture inventory"
  INVENTORY="${FIXTURES_DIR}/surface-inventory.yaml"
  [[ -f "${INVENTORY}" ]] || { log "fixture missing: ${INVENTORY}"; exit 1; }
fi

python3 - <<PYEOF
import yaml, os, sys

repo_root = '${REPO_ROOT}'
inventory_path = '${INVENTORY}'
audit_mode = '${AUDIT_MODE}'

if not os.path.isfile(inventory_path):
    if audit_mode == 'dry':
        print('[9.2] dry: inventory file not found, skipping', file=sys.stderr)
        sys.exit(0)
    print(f'[9.2] FAIL: inventory not found: {inventory_path}', file=sys.stderr)
    sys.exit(1)

inventory = yaml.safe_load(open(inventory_path))
failures = []
checked = 0

for s in inventory['surfaces']:
    name = s['name']
    svc_dir = os.path.join(repo_root, s['dir'])
    framework = s['framework']

    # Determine artefact indicator
    if framework == 'flutter':
        indicator = os.path.join(svc_dir, 'pubspec.yaml')
    elif framework == 'tauri':
        indicator = os.path.join(svc_dir, 'src-tauri', 'tauri.conf.json')
    elif framework in ('astro', 'liveview', 'django'):
        indicator = svc_dir  # directory existence is sufficient
    else:
        indicator = svc_dir

    exists = os.path.exists(indicator)
    if exists:
        checked += 1
    else:
        failures.append(f'{name}: missing {indicator}')
        print(f'[9.2] FAIL: {name} artefact not found: {indicator}', file=sys.stderr)

if failures:
    print(f'[9.2] {len(failures)} surface(s) missing deployable artefact', file=sys.stderr)
    sys.exit(1)

print(f'[9.2] PASS: {checked} surfaces have deployable artefacts', file=sys.stderr)
PYEOF

log "PASS: surface inventory complete"
