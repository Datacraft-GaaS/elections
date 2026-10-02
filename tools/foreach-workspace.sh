#!/usr/bin/env bash
# Run a command in every Rust workspace root in the repo.
#
# `just` recipes cannot loop over the output of another recipe: `$$(...)` reaches
# the shell as a literal `$$` (syntax error) and `$(...)` is expanded by `just`
# itself, splicing multi-line output into the loop word and corrupting it. This
# script owns the loop and its exit status, so callers get real failure
# propagation instead of a subshell that cannot fail the caller.
#
# Usage: foreach-workspace.sh <label> <command...>
#   label   — echoed as "[<label>] workspace: <path>" for progress
#   command — run with $EEL_WS set to each workspace manifest path
#
# Workspaces whose manifest declares no members are skipped, matching the
# behaviour the recipes documented.

set -uo pipefail

label="${1:?usage: foreach-workspace.sh <label> <command...>}"
shift || true

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

status=0

# `mapfile` is bash 4+; macOS ships bash 3.2, so read the list with a while loop.
workspaces="$(
  echo "$root/Cargo.toml"
  find "$root/services" -maxdepth 2 -name Cargo.toml -exec grep -l '^\[workspace\]' {} \; 2>/dev/null
)"

while IFS= read -r ws; do
  [ -n "$ws" ] || continue
  [ -f "$ws" ] || continue

  if ! cargo metadata --no-deps --quiet --manifest-path "$ws" 2>/dev/null |
    python3 -c "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then
    echo "[$label] workspace ${ws#"$root"/} has no members, skipping"
    continue
  fi

  echo "[$label] workspace: ${ws#"$root"/}"
  export EEL_WS="$ws"
  if ! "$@"; then
    echo "[$label] FAILED in ${ws#"$root"/}" >&2
    status=1
  fi
done <<EOF
$workspaces
EOF

exit "$status"