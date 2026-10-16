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
# Apply every SQL migration the repo ships, in the order tools/eel-migrate
# discovers them, to a throwaway Postgres. A migration that cannot be applied is
# a schema the deploy never creates -- and nothing else catches that, because
# the migrations are raw SQL with no ORM to compare against.
#
# Requires DATABASE_URL, pointing at a database this script may drop and
# recreate. It refuses to run against a database whose name does not end in
# `_test` or `_mig_test`, so an unset or mistyped URL fails closed.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ -z "${DATABASE_URL:-}" ]; then
	echo "check-migrations: DATABASE_URL is not set" >&2
	exit 1
fi

dbname="${DATABASE_URL##*/}"
dbname="${dbname%%\?*}"
case "$dbname" in
*_test | *_mig_test) ;;
*)
	echo "check-migrations: refusing to run against database '$dbname'" >&2
	echo "  it must end in _test or _mig_test; this script drops and recreates it" >&2
	exit 1
	;;
esac

# Discover exactly as tools/eel-migrate does: services/*/migrations/*.sql and
# services/*/crates/*/migrations/*.sql, sorted by (service, version, name).
# A .sql migration anywhere else is invisible to the deploy, so it is reported
# rather than silently skipped.
#
# Classify by the path's *shape*, not by a find -depth: eel-migrate builds its
# walk from directory names, and depth arithmetic silently mis-counts the
# crate-level form (services/<svc>/crates/<crate>/migrations/ is six
# components deep, not four).
discover() {
	python3 - "$ROOT" "$1" <<'PY'
import os, re, sys
root, mode = sys.argv[1], sys.argv[2]
svc = re.compile(r"^services/([^/]+)/migrations/([^/]+)\.sql$")
crate = re.compile(r"^services/([^/]+)/crates/([^/]+)/migrations/([^/]+)\.sql$")

def version_of(name):
    # Mirrors version_of() in tools/eel-migrate/src/main.rs: the first run of
    # digits anywhere in the stem. `V4` is 4 and `V056` is 56, so V4 sorts
    # first -- alphabetical order would put V056 first and apply an ALTER to a
    # table V4 has not created yet.
    digits = re.match(r"[^0-9]*([0-9]+)", name)
    return int(digits.group(1)) if digits else 0

rows = []
for dirpath, dirnames, filenames in os.walk(os.path.join(root, "services")):
    dirnames[:] = [d for d in dirnames if d not in ("target", ".venv", "node_modules")]
    for fn in filenames:
        if not fn.endswith(".sql"):
            continue
        rel = os.path.relpath(os.path.join(dirpath, fn), root)
        # Only files inside a `migrations/` directory are migrations. Report
        # query templates (`reports-templates/sql/`) and seeds (`data/seeds/`)
        # are neither applied nor expected to be, so flagging them would be a
        # checker reporting on a class it does not own.
        if "migrations" not in rel.split(os.sep)[:-1]:
            continue
        m = svc.match(rel) or crate.match(rel)
        if (mode == "known") == bool(m):
            if m:
                service = m.group(1) if m.re is svc else f"{m.group(1)}/{m.group(2)}"
            else:
                service = rel
            rows.append((service, version_of(fn), fn, os.path.join(root, rel)))

# eel-migrate sorts by (service, version, name); match it so a failure here
# means the same thing it would at deploy time.
rows.sort(key=lambda r: (r[0], r[1], r[2]))
print("\n".join(r[3] for r in rows))
PY
}

discovered="$(discover known)"
all_sql="$(discover all)"

orphans="$(comm -13 <(printf '%s\n' "$discovered") <(printf '%s\n' "$all_sql"))"
if [ -n "$orphans" ]; then
	echo "check-migrations: these .sql files are never applied -- eel-migrate does" >&2
	echo "  not look here. Move them under services/<svc>/migrations/:" >&2
	printf '  %s\n' $orphans >&2
	exit 1
fi

count="$(printf '%s\n' "$discovered" | grep -c . || true)"
if [ "$count" -eq 0 ]; then
	echo "check-migrations: found no migrations -- the walk is wrong" >&2
	exit 1
fi

# Connect to `postgres` for the DDL: a session cannot drop the database it is
# connected to.
psql -d postgres -q -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS \"$dbname\"" >/dev/null 2>&1
if ! psql -d postgres -q -v ON_ERROR_STOP=1 -c "CREATE DATABASE \"$dbname\"" >/dev/null 2>&1; then
	echo "check-migrations: cannot drop/create $dbname at $DATABASE_URL" >&2
	exit 1
fi

fail=0
applied=0
while IFS= read -r m; do
	[ -n "$m" ] || continue
	if psql -d "$dbname" -q -v ON_ERROR_STOP=1 -f "$m" >/tmp/check-migrations.out 2>&1; then
		applied=$((applied + 1))
	else
		fail=$((fail + 1))
		printf 'FAIL %s\n' "${m#"$ROOT"/}"
		sed -n '1,4p' /tmp/check-migrations.out | sed 's/^/     /'
	fi
done <<<"$discovered"

if [ "$fail" -ne 0 ]; then
	echo "check-migrations: $fail of $count migrations do not apply" >&2
	exit 1
fi

echo "check-migrations: $applied migrations applied clean"
