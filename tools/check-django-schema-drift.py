#!/usr/bin/env python3
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
"""Fail when a Django model disagrees with the SQL schema that creates its table.

The repository has two schema sources. `services/<svc>/migrations/*.sql` is
applied by tools/eel-migrate at deploy; the Django models in
`services/<svc>/django/<app>` describe the same `db_table` names independently.
Nothing compared them, so 68 columns diverged across 10 apps without a single
gate noticing.

The rule, and why it is per table rather than per app:

- A table some migration CREATEs is SQL-owned. The deploy creates it, so the
  model must describe it exactly; a model column the table lacks raises
  `ProgrammingError` the first time the ORM touches it.
- A table no migration creates is model-owned. Django's `run_syncdb` builds it
  from the model, and it is the model that is authoritative.

Requires DATABASE_URL pointing at a database this script may drop and recreate,
whose name ends in `_test`, `_mig_test` or `_drift_test`.

Usage: check-django-schema-drift.py [--root DIR] [--allowlist FILE]
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "lib"))
from discover_migrations import discover  # noqa: E402

# Django builds its own tables from its own migrations; they are not ours to
# reconcile.
IGNORED_TABLE_PREFIXES = ("django_", "auth_")

# These apps cannot load far enough to be introspected. Each entry is a
# documented blocker, not a pass -- see the `reason`.
KNOWN_UNLOADABLE = {
    "hello-svc/django/hello_project": (
        "placeholder service; its models are a smoke-test fixture, and its "
        "settings hardcode SQLite with no Postgres path"
    ),
}


def split_url(url: str) -> tuple[str, str]:
    """Return (maintenance_url, dbname) for a Postgres connection URL.

    The maintenance URL is the same connection with the database replaced by
    `postgres`, so DROP/CREATE target the server the URL names. Passing only
    `-d postgres` to psql connects to whatever the default target is -- a
    different server, where this would happily build a schema and inspect it
    while the URL pointed somewhere else.
    """
    dbname = url.rsplit("/", 1)[-1].split("?")[0]
    maintenance = url.rsplit("/", 1)[0] + "/postgres"
    return maintenance, dbname


def load_schema(psql: str, url: str, root: str) -> dict[str, dict[str, str]]:
    """Build a database from the migration chain and return {table: {column: type}}."""
    maintenance, dbname = split_url(url)
    for stmt in (f'DROP DATABASE IF EXISTS "{dbname}"', f'CREATE DATABASE "{dbname}"'):
        subprocess.run(
            [psql, maintenance, "-q", "-v", "ON_ERROR_STOP=1", "-c", stmt],
            check=True,
            capture_output=True,
        )

    migrations = discover(root, "known")
    if not migrations:
        print("check-django-schema-drift: found no migrations -- the walk is wrong", file=sys.stderr)
        raise SystemExit(1)

    for _service, _version, _name, path in migrations:
        proc = subprocess.run(
            [psql, url, "-q", "-v", "ON_ERROR_STOP=1", "-f", path],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            print(f"check-django-schema-drift: migration does not apply: {path}", file=sys.stderr)
            print(proc.stderr.strip()[:400], file=sys.stderr)
            raise SystemExit(1)

    # `format_type` rather than `data_type`: data_type collapses every
    # user-defined and array type to "ARRAY" or "USER-DEFINED", and an array's
    # element type is exactly what a JSONField-vs-ArrayField mistake gets wrong.
    # Postgres builds the JSON: a type can contain a comma (`numeric(10,2)`), so
    # splitting a joined string on delimiters parses some tables and not others.
    out = subprocess.run(
        [
            psql, url, "-tAc",
            "select json_object_agg(t, cols) from ("
            "  select c.table_name as t,"
            "         json_object_agg(c.column_name, format_type(a.atttypid, a.atttypmod)) as cols"
            "  from information_schema.columns c"
            "  join pg_class cl on cl.relname = c.table_name and cl.relnamespace ="
            "    (select oid from pg_namespace where nspname = 'public')"
            "  join pg_attribute a on a.attrelid = cl.oid and a.attname = c.column_name"
            "  where c.table_schema = 'public'"
            "  group by c.table_name"
            ") s",
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return json.loads(out)


def load_models(app: pathlib.Path) -> dict[str, dict[str, str]] | None:
    """Return {db_table: {column: type}} for one Django app, or None if it cannot load."""
    settings = sorted(p.name for p in app.iterdir() if (p / "settings.py").exists())
    if not settings:
        return None
    # The model's types are asked of a Postgres backend built from the app's own
    # settings, not of `connection`. Several apps here hardcode SQLite, and
    # SQLite's backend spells the same field `char(32)` and `datetime` where
    # Postgres says `uuid` and `timestamp with time zone` -- comparing those
    # against the migration would report every UUID and timestamp column as
    # drift. The backend answers `db_type` without connecting, so this needs no
    # live database.
    code = (
        "import os,django,json;"
        f"os.environ.setdefault('DJANGO_SETTINGS_MODULE',{settings[0] + '.settings'!r});"
        "django.setup();"
        "from django.conf import settings as S;"
        "from django.db.backends.postgresql.base import DatabaseWrapper;"
        "from django.apps import apps as A;"
        "sd = dict(S.DATABASES['default']);"
        "sd.update(ENGINE='django.db.backends.postgresql', NAME='x', USER='x',"
        " PASSWORD='', HOST='127.0.0.1', PORT='55993', OPTIONS={}, TIME_ZONE=None,"
        " CONN_MAX_AGE=0, CONN_HEALTH_CHECKS=False, AUTOCOMMIT=True,"
        " ATOMIC_REQUESTS=False, TEST={}, DISABLE_SERVER_SIDE_CURSORS=False);"
        "pg = DatabaseWrapper(sd, alias='pg');"
        # `f.column`, not `f.name`: an FK's name is `course` and its column is
        # `course_id`, so matching on names reports every FK as missing.
        # `db_type` is what Django would emit in CREATE TABLE, which is the
        # comparable form of the migration's own column type.
        "print(json.dumps({m._meta.db_table:"
        "{f.column: (f.db_type(pg) or '') "
        "for f in m._meta.get_fields() if getattr(f,'column',None)}"
        " for m in A.get_models()}))"
    )
    env = {**os.environ, "DJANGO_TEST_SQLITE": "true"}
    env.pop("DATABASE_URL", None)
    try:
        proc = subprocess.run(
            # `--with psycopg` overlays the driver for this invocation only.
            # Building a Postgres backend needs it, several apps do not declare
            # it (they hardcode SQLite), and installing it into their venvs
            # would edit the apps to suit the checker.
            ["uv", "run", "--with", "psycopg[binary]>=3.1", "python", "-c", code],
            cwd=app, capture_output=True, text=True, timeout=300, env=env,
        )
    except subprocess.TimeoutExpired:
        return None
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout.strip().splitlines()[-1])
    except (json.JSONDecodeError, IndexError):
        return None


# The text family: Postgres treats these as one type for every purpose the ORM
# cares about, and the migration chain and the models disagree freely between
# them (`TEXT` in SQL, `varchar(50)` from a CharField). Flagging that pair would
# bury the findings that raise.
_TEXT_FAMILY = re.compile(
    r"^(text|character varying(\(\d+\))?|varchar(\(\d+\))?|character(\(\d+\))?|char(\(\d+\))?)$"
)


def type_equivalent(sql_type: str, model_type: str) -> bool:
    """Whether two type spellings describe the same storage class.

    Deliberately narrow. It normalizes only the text family, where SQL says
    `text` and a CharField says `varchar(n)` -- a difference in length
    enforcement, not in what the column holds. Everything else must match
    exactly, because everything else that differs (jsonb against text[], text
    against uuid) raises `ProgrammingError` on the first statement.
    """
    # Postgres renders `numeric(5,2)`, Django's backend `numeric(5, 2)`. That
    # space is the only difference between them, and a comparison that keeps it
    # reports a correct column as drift. Only the comma's spacing is touched:
    # collapsing all whitespace would rewrite `character varying` out of the
    # text family below.
    sql_type = re.sub(r",\s*", ",", sql_type.strip().lower())
    model_type = re.sub(r",\s*", ",", model_type.strip().lower())
    if sql_type == model_type:
        return True
    if sql_type.endswith("[]") and model_type.endswith("[]"):
        return type_equivalent(sql_type[:-2], model_type[:-2])
    return bool(_TEXT_FAMILY.match(sql_type) and _TEXT_FAMILY.match(model_type))


def find_apps(root: pathlib.Path) -> list[pathlib.Path]:
    return sorted(
        mp.parent
        for mp in root.glob("services/*/django/*/manage.py")
        if ".venv" not in str(mp)
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=str(pathlib.Path(__file__).resolve().parent.parent))
    ap.add_argument("--allowlist", default=None)
    args = ap.parse_args()
    root = pathlib.Path(args.root).resolve()

    url = os.environ.get("DATABASE_URL", "")
    if not url:
        print("check-django-schema-drift: DATABASE_URL is not set", file=sys.stderr)
        return 1
    dbname = url.rsplit("/", 1)[-1].split("?")[0]
    if not dbname.endswith(("_test", "_mig_test", "_drift_test")):
        print(
            f"check-django-schema-drift: refusing to run against '{dbname}'; "
            "it must end in _test, _mig_test or _drift_test (this script drops it)",
            file=sys.stderr,
        )
        return 1
    psql = os.environ.get("PSQL", "psql")

    allowlist: dict[str, str] = {}
    if args.allowlist and pathlib.Path(args.allowlist).exists():
        allowlist = json.loads(pathlib.Path(args.allowlist).read_text())

    schema = load_schema(psql, url, str(root))
    if not schema:
        print("check-django-schema-drift: schema is empty -- the dump is wrong", file=sys.stderr)
        return 1

    apps = find_apps(root)
    if not apps:
        print("check-django-schema-drift: found no Django apps -- the walk is wrong", file=sys.stderr)
        return 1

    findings: list[str] = []
    unloadable: list[str] = []
    checked = 0
    for app in apps:
        rel = str(app.relative_to(root)).removeprefix("services/")
        models = load_models(app)
        if models is None:
            if rel not in KNOWN_UNLOADABLE:
                unloadable.append(rel)
            continue
        checked += 1
        for table, cols in models.items():
            if table.startswith(IGNORED_TABLE_PREFIXES):
                continue
            if table not in schema:
                # Model-owned: no migration creates it, so Django's run_syncdb
                # builds it and the model is authoritative.
                continue
            key = f"{rel}:{table}"
            if key in allowlist:
                continue
            sql_cols = schema[table]
            missing = sorted(set(cols) - set(sql_cols))
            if missing:
                findings.append(f"  {rel}\n    {table}\n      no such column: {', '.join(missing)}")
                continue
            # Name agreement is not schema agreement. `access_zones` was TEXT[]
            # in the migration and JSONField in the model: every name matched,
            # and the ORM raised on the first insert.
            wrong_type = [
                f"{c} is {cols[c]} in the model, {sql_cols[c]} in the table"
                for c in sorted(cols)
                if sql_cols[c] != cols[c] and not type_equivalent(sql_cols[c], cols[c])
            ]
            if wrong_type:
                findings.append(f"  {rel}\n    {table}\n      " + "\n      ".join(wrong_type))

    if unloadable:
        print("check-django-schema-drift: these apps do not load:", file=sys.stderr)
        for a in unloadable:
            print(f"  {a}", file=sys.stderr)
        return 1

    if findings:
        print(
            "check-django-schema-drift: models name columns the migrations do not create.",
            file=sys.stderr,
        )
        print(
            "  Either fix the model to match the table, or add a migration that",
            file=sys.stderr,
        )
        print("  creates the column. Both sides are load-bearing.\n", file=sys.stderr)
        for f in findings:
            print(f, file=sys.stderr)
        print(f"\n{len(findings)} table(s) drifted", file=sys.stderr)
        return 1

    print(f"check-django-schema-drift: {checked} apps, {len(schema)} tables, no drift")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
