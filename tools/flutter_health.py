#!/usr/bin/env python3
"""Measure analyzer results across every Dart/Flutter package, honestly.

The previous harness counted occurrences of "error" in `flutter analyze` output.
That silently reported a package whose `flutter pub get` had failed as having
zero errors, because analyze prints the pub failure instead of analysis results
and the grep matched nothing. A package that cannot even resolve its
dependencies was being scored as clean.

This harness distinguishes three outcomes and never conflates them:

  clean       - analyze ran and found no errors
  errors      - analyze ran and found N errors
  UNMEASURABLE - `flutter pub get` failed, so nothing was analysed

An unmeasurable package is a failure of the build, not a pass.
"""

import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))
ERROR_RE = re.compile(r"^\s*error •", re.M)
WARN_RE = re.compile(r"^\s*warning •", re.M)
INFO_RE = re.compile(r"^\s*info •", re.M)
# "1 issue found" is singular, so matching the plural alone hides any
# package with exactly one finding.
SUMMARY_RE = re.compile(r"\bissues? found", re.I)


SKIP = {".dart_tool", "build", "node_modules", ".git", "target"}


def package_dirs(targets):
    found = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in SKIP]
            if "pubspec.yaml" in filenames:
                found.append(dirpath)
    return sorted(found)


def main():
    targets = sys.argv[1:] or ["."]
    dirs = package_dirs(targets)

    total_errors = 0
    total_warnings = 0
    total_infos = 0
    rows = []
    unmeasurable = []

    for path in dirs:
        rel = os.path.relpath(path, ROOT)
        pub = subprocess.run(["flutter", "pub", "get"], cwd=path,
                             capture_output=True, text=True, timeout=600)
        if pub.returncode != 0:
            tail = [l for l in pub.stderr.splitlines() if l.strip()][-1:] or ["(no detail)"]
            unmeasurable.append((rel, tail[0][:100]))
            rows.append((rel, None, None, None))
            continue

        try:
            result = subprocess.run(["flutter", "analyze"], cwd=path,
                                    capture_output=True, text=True, timeout=900)
        except subprocess.TimeoutExpired:
            unmeasurable.append((rel, "analyze timed out"))
            rows.append((rel, None, None, None))
            continue

        output = result.stdout + result.stderr
        # "No issues found!" means analyze really ran.
        if "No issues found" in output:
            errors, warnings, infos = 0, 0, 0
        elif ERROR_RE.search(output) or SUMMARY_RE.search(output):
            errors = len(ERROR_RE.findall(output))
            warnings = len(WARN_RE.findall(output))
            infos = len(INFO_RE.findall(output))
        else:
            unmeasurable.append((rel, "analyze produced no parseable output"))
            rows.append((rel, None, None, None))
            continue

        total_errors += errors
        total_warnings += warnings
        total_infos += infos
        rows.append((rel, errors, warnings, infos))

    for rel, errors, warnings, infos in rows:
        if errors is None:
            print(f"  {rel:<52} UNMEASURABLE")
        elif errors or warnings or infos:
            print(f"  {rel:<52} errors={errors:<4} warnings={warnings:<4} infos={infos}")

    measured = sum(1 for _, e, _, _ in rows if e is not None)
    print()
    print(f"packages: {len(rows)}  measured: {measured}  unmeasurable: {len(unmeasurable)}")
    print(f"errors: {total_errors}  warnings: {total_warnings}  infos: {total_infos}")
    for rel, why in unmeasurable:
        print(f"  UNMEASURABLE {rel}: {why}")
    return 0 if not unmeasurable and total_errors == 0 and total_warnings == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
