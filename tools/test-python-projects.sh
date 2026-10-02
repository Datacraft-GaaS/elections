#!/usr/bin/env bash
# Run the `tests/ci` suite of every Python project under services/.
#
# Kept out of the justfile for the same reason as foreach-workspace.sh: a long
# in-line loop in a recipe is fragile, and pytest output on stdout can corrupt
# any variable the recipe is tracking by hand.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

list="$(mktemp)"
trap 'rm -f "$list"' EXIT

find "$root/services" -name pyproject.toml -maxdepth 5 -exec dirname {} \; > "$list"

rc=0
ran=0

while IFS= read -r dir; do
  [ -n "$dir" ] || continue
  [ -d "$dir/tests/ci" ] || continue
  ran=1

  echo "[test-unit] python: ${dir#"$root"/}"

  # Projects without an optional-dependencies table have no `dev` extra, and
  # passing --extra dev there is a hard error.
  extra=()
  if grep -q "optional-dependencies" "$dir/pyproject.toml" 2>/dev/null; then
    extra=(--extra dev)
  fi

  (
    cd "$dir" || exit 1
    uv sync --quiet "${extra[@]+"${extra[@]}"}" || exit 1
    # 5 = no tests collected. An empty tests/ci is a placeholder project, not a
    # failure; without this the harness reports a red build for a dir of nothing.
    uv run pytest -q tests/ci
    status=$?
    [ "$status" -eq 5 ] && exit 0
    exit "$status"
  ) || rc=1
done < "$list"

[ "$ran" -eq 1 ] || echo "[test-unit] no Python projects with tests/ci, skipping"

exit "$rc"
