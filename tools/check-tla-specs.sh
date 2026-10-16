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
#
# Lint every TLA+ module under formal/ with SANY, the reference front end.
#
# Why SANY and not `tlc`: TLC needs a .cfg per spec and only exists for a
# handful of them, so it would silently skip most of the corpus. SANY parses
# and resolves every module, needs no config, and is what TLC invokes anyway.
#
# Why the exit code is not the signal: SANY exits 0 on a module with thirteen
# undefined symbols. The signal is the pair of lines it prints -- on success,
# "Linting of module X" and no `*** Errors`; on failure, an error count and no
# linting line. A run that prints neither is a failure too: SANY did not get
# far enough to judge the spec.
#
# Canary this gate before trusting it: point it at a module with a hyphen in
# the name, or one whose filename disagrees with its module, and watch it fail.
set -uo pipefail

JAR="${TLA_JAR:-$HOME/.cache/tla2tools.jar}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v java >/dev/null 2>&1; then
	echo "check-tla-specs: java not on PATH; cannot run SANY" >&2
	exit 1
fi
if [ ! -f "$JAR" ]; then
	echo "check-tla-specs: SANY jar not found at $JAR" >&2
	echo "  curl -L -o $JAR https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar" >&2
	exit 1
fi

fail=0
count=0
while IFS= read -r spec; do
	count=$((count + 1))
	out="$(java -cp "$JAR" tla2sany.SANY "$spec" 2>&1)"
	errors="$(printf '%s\n' "$out" | grep -o '\*\*\* Errors: [0-9]*' | tail -1)"
	if printf '%s\n' "$out" | grep -q "Linting of module" && [ -z "$errors" ]; then
		continue
	fi
	fail=$((fail + 1))
	detail="$errors"
	if [ -z "$detail" ]; then
		detail="no error count and no linting line"
	fi
	first="$(printf '%s\n' "$out" | grep -m1 -E 'Unknown operator|Could not find declaration|Couldn.t resolve|Was expecting|File name|followed by action' || true)"
	printf 'FAIL %s: %s\n' "${spec#"$ROOT"/}" "$detail"
	[ -n "$first" ] && printf '     %s\n' "$first"
done < <(find "$ROOT/formal" -name '*.tla' | sort)

if [ "$count" -eq 0 ]; then
	echo "check-tla-specs: found no .tla files under $ROOT/formal -- the walk is wrong" >&2
	exit 1
fi

if [ "$fail" -ne 0 ]; then
	echo "check-tla-specs: $fail of $count specs do not verify clean" >&2
	exit 1
fi

echo "check-tla-specs: $count specs lint clean"
