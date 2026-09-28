#!/usr/bin/env python3
"""Remove imports the analyzer reports as unused, then re-analyze.

A previous pass guessed at missing stdlib imports by scanning for bare
identifiers, which over-matched and left imports the analyzer then flagged as
unused. The analyzer is authoritative here, so this reads its diagnostics
rather than re-deriving them, and only ever deletes an import line the analyzer
has named.
"""

import re
import subprocess
import sys

ROOT = "/Users/nyimbiodero/src/pjs/eel"
UNUSED = re.compile(r"^warning • Unused import: '([^']+)'.*• (\S+\.dart):(\d+):\d+")


def analyse(pkg):
    r = subprocess.run(["flutter", "analyze"], cwd=pkg, capture_output=True,
                       text=True, timeout=900)
    return r.stdout + r.stderr


def main():
    pkgs = sys.argv[1:]
    total_removed = 0
    for pkg in pkgs:
        for _ in range(6):  # a removal can expose another; iterate
            out = analyse(pkg)
            found = {}
            for line in out.splitlines():
                m = UNUSED.match(line.strip())
                if m:
                    found.setdefault(m.group(2), set()).add(m.group(1))
            if not found:
                break
            for rel, imports in found.items():
                path = f"{pkg}/{rel}"
                lines = open(path).read().split("\n")
                keep = [l for l in lines
                        if not any(l.strip() == f"import '{i}';" for i in imports)]
                if len(keep) != len(lines):
                    open(path, "w").write("\n".join(keep))
                    total_removed += len(lines) - len(keep)
        print(f"{pkg}: warnings now {analyse(pkg).count('warning •')}")
    print(f"removed {total_removed} unused import lines")


if __name__ == "__main__":
    main()
