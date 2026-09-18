#!/usr/bin/env bash
# common.sh — Shared helpers for all mobile scripts.
# Sourced by every script in scripts/mobile/. Not executable directly.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
APP_REGISTRY="${SCRIPT_DIR}/apps.txt"

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------
_log_info()  { echo "[INFO]  $*"; }
_log_warn()  { echo "[WARN]  $*" >&2; }
_log_error() { echo "[ERROR] $*" >&2; }

# ---------------------------------------------------------------------------
# App registry lookup
# Format: app_name|path|flavors|targets|mdm_app_id
#   app_name   — slug used in scripts (e.g. poll-clerk)
#   path       — relative to REPO_ROOT (e.g. services/field-mobile-svc/apps/poll-clerk)
#   flavors    — comma-separated: dev,staging,production
#   targets    — comma-separated: android,ios
#   mdm_app_id — MDM application identifier (empty if N/A)
# ---------------------------------------------------------------------------
get_app_path() {
  local app="$1"
  [[ -f "$APP_REGISTRY" ]] || { _log_error "app registry not found: $APP_REGISTRY"; return 1; }
  awk -F'|' -v a="$app" '$1 == a {print $2; exit}' "$APP_REGISTRY"
}

get_app_flavors() {
  local app="$1"
  awk -F'|' -v a="$app" '$1 == a {print $2; exit}' "$APP_REGISTRY"
}

# ---------------------------------------------------------------------------
# Flutter version check (reads .flutter-version if present)
# ---------------------------------------------------------------------------
check_flutter_version() {
  local expected=""
  if [[ -f "${REPO_ROOT}/.flutter-version" ]]; then
    expected="$(cat "${REPO_ROOT}/.flutter-version" | tr -d '[:space:]')"
  fi
  if [[ -n "$expected" ]]; then
    local installed
    installed="$(flutter --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
    if [[ "$installed" != "$expected" ]]; then
      _log_warn "Flutter version mismatch: expected $expected, got $installed"
    else
      _log_info "Flutter version OK: $installed"
    fi
  else
    _log_info "No .flutter-version pin found; using system Flutter"
  fi
}

# ---------------------------------------------------------------------------
# Signing material fetch from OpenBao (dev only)
# ---------------------------------------------------------------------------
fetch_signing_material() {
  local env="$1"
  if ! command -v vault &>/dev/null; then
    _log_warn "vault not found; skipping signing material fetch"
    return 0
  fi
  local key_props="${REPO_ROOT}/android/key.properties"
  if [[ -f "$key_props" ]]; then
    _log_warn "key.properties already exists at $key_props — skipping fetch"
    return 0
  fi
  _log_info "Fetching signing material for $env from OpenBao..."
  local store_pass key_pass alias_name
  store_pass="$(vault kv get -field=store_password "secret/eel/mobile/signing/$env" 2>/dev/null || echo "")"
  key_pass="$(vault kv get -field=key_password "secret/eel/mobile/signing/$env" 2>/dev/null || echo "")"
  alias_name="$(vault kv get -field=alias "secret/eel/mobile/signing/$env" 2>/dev/null || echo "")"
  if [[ -n "$store_pass" && -n "$key_pass" ]]; then
    cat > "$key_props" <<EOF
storePassword=$store_pass
keyPassword=$key_pass
keyAlias=$alias_name
storeFile=../app/debug.keystore
EOF
    _log_info "Wrote $key_props (not committed — add to .gitignore)"
  else
    _log_warn "No signing material returned for $env; APK builds will use debug keys"
  fi
}

# ---------------------------------------------------------------------------
# Git helpers
# ---------------------------------------------------------------------------
git_sha() { git rev-parse --short HEAD 2>/dev/null || echo "unknown"; }
git_epoch() { git log -1 --format=%ct 2>/dev/null || echo "0"; }

# ---------------------------------------------------------------------------
# App iteration (used by all-* variants)
# ---------------------------------------------------------------------------
iterate_apps() {
  local filter="${1:-}"
  while IFS='|' read -r app_name app_path flavors targets mdm_id; do
    [[ "$app_name" =~ ^# ]] && continue
    [[ -z "$app_path" ]] && continue
    if [[ -n "$filter" && "$app_name" != "$filter" ]]; then
      continue
    fi
    echo "${app_name}|${app_path}|${flavors}|${targets}|${mdm_id}"
  done < "$APP_REGISTRY"
}
