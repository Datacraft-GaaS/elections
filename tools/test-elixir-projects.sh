#!/usr/bin/env bash
# Run `mix test` for every Elixir project under services/.
#
# Also a script rather than an in-line recipe loop: there are 169 mix projects,
# which overflows a single xargs command line.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

list="$(mktemp)"
trap 'rm -f "$list"' EXIT

find "$root/services" -name mix.exs -maxdepth 6 -exec dirname {} \; > "$list"

if [ ! -s "$list" ]; then
  echo "[test-unit] no Elixir projects, skipping"
  exit 0
fi

rc=0
while IFS= read -r dir; do
  [ -n "$dir" ] || continue
  echo "[test-unit] elixir: ${dir#"$root"/}"
  ( cd "$dir" && mix test --no-color ) || rc=1
done < "$list"

exit "$rc"
