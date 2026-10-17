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
"""Discover SQL migrations exactly as tools/eel-migrate does.

One copy, because two checkers that walk migrations independently will
eventually disagree, and the one that disagrees silently is the one whose
verdict you read.

Usage: discover_migrations.py <root> [known|all]

`known` prints the paths eel-migrate applies; `all` additionally prints every
other .sql under a `migrations/` directory, which is the set that is silently
never applied. Both go to stdout, one absolute path per line, in the order
eel-migrate applies them.
"""

from __future__ import annotations

import os
import re
import sys

# eel-migrate builds its walk from directory names, so match the path's shape
# rather than counting depth: services/<svc>/crates/<crate>/migrations/ is six
# components deep, not four, and depth arithmetic mis-classifies it.
SVC = re.compile(r"^services/([^/]+)/migrations/([^/]+)\.sql$")
CRATE = re.compile(r"^services/([^/]+)/crates/([^/]+)/migrations/([^/]+)\.sql$")

SKIP_DIRS = frozenset({"target", ".venv", "node_modules", "__pycache__"})


def version_of(name: str) -> int:
    """First run of digits anywhere in the stem.

    Mirrors version_of() in tools/eel-migrate/src/main.rs. `V4` is 4 and `V056`
    is 56, so V4 sorts first -- alphabetical order would put V056 first and
    apply an ALTER to a table V4 has not created yet.
    """
    digits = re.match(r"[^0-9]*([0-9]+)", name)
    return int(digits.group(1)) if digits else 0


def discover(root: str, mode: str) -> list[tuple[str, int, str, str]]:
    """Return (service, version, filename, absolute path) sorted as eel-migrate sorts."""
    rows: list[tuple[str, int, str, str]] = []
    for dirpath, dirnames, filenames in os.walk(os.path.join(root, "services")):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if not fn.endswith(".sql"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, fn), root)
            # Only a file inside a `migrations/` directory is a migration.
            # Report query templates (`reports-templates/sql/`) and seeds
            # (`data/seeds/`) are neither applied nor expected to be.
            if "migrations" not in rel.split(os.sep)[:-1]:
                continue
            m = SVC.match(rel) or CRATE.match(rel)
            if (mode == "known") != bool(m):
                continue
            if m:
                service = m.group(1) if m.re is SVC else f"{m.group(1)}/{m.group(2)}"
            else:
                service = rel
            rows.append((service, version_of(fn), fn, os.path.join(root, rel)))
    rows.sort(key=lambda r: (r[0], r[1], r[2]))
    return rows


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    root = sys.argv[1]
    mode = sys.argv[2] if len(sys.argv) > 2 else "known"
    if mode not in ("known", "all"):
        print(f"discover_migrations: unknown mode {mode!r}", file=sys.stderr)
        return 2
    for row in discover(root, mode):
        print(row[3])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
