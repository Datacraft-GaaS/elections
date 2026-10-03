#!/usr/bin/env bash
# Run `mix test` for every Elixir project under services/.
#
# Also a script rather than an in-line recipe loop: there are 169 mix projects,
# which overflows a single xargs command line.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Homebrew installs Erlang unlinked, so `mix` is on PATH but `erl` is not, and
# every rebar3 dependency fails to compile with "exec: erl: not found".
if ! command -v erl >/dev/null 2>&1 && [ -x /opt/homebrew/opt/erlang/bin/erl ]; then
  PATH="/opt/homebrew/opt/erlang/bin:$PATH"
  export PATH
fi

list="$(mktemp)"
trap 'rm -f "$list"' EXIT

find "$root/services" -name mix.exs -maxdepth 6 \
  -not -path '*/deps/*' -not -path '*/_build/*' \
  -exec dirname {} \; > "$list"

if [ ! -s "$list" ]; then
  echo "[test-unit] no Elixir projects, skipping"
  exit 0
fi

rc=0
while IFS= read -r dir; do
  [ -n "$dir" ] || continue
  # A project with no test helper has no suite; mix treats that as an error.
  [ -f "$dir/test/test_helper.exs" ] || continue
  echo "[test-unit] elixir: ${dir#"$root"/}"
  (
    cd "$dir" || exit 1
    # Without this, a project whose deps were never fetched fails with
    # "the dependency is not available, run mix deps.get".
    mix deps.get --quiet >/dev/null 2>&1 || true
    mix test --no-color
  ) || rc=1
done < "$list"

exit "$rc"
