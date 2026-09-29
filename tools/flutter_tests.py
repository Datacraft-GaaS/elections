#!/usr/bin/env python3
"""Run every Dart/Flutter package's tests, reporting per-package pass counts.

Like tools/flutter_health.py, this refuses to score a package it could not
measure. A package whose tests failed to compile is a failure, not a pass, and
a package with no test directory is reported as such rather than skipped
silently.
"""

import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))
COUNT = re.compile(r"\+(\d+)(?:\s+-(\d+))?:\s*(?:All tests passed|Some tests failed)")


def package_dirs(targets):
    out = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames
                           if d not in (".dart_tool", "build", "node_modules")]
            if "pubspec.yaml" in filenames:
                out.append(dirpath)
    return sorted(out)


def main():
    targets = sys.argv[1:] or ["services", "packages"]
    dirs = package_dirs(targets)
    total_pass = total_fail = 0
    clean, failed, untested, broken, empty = [], [], [], [], []

    for path in dirs:
        rel = os.path.relpath(path, ROOT)
        testdir = os.path.join(path, "test")
        if not os.path.isdir(testdir):
            untested.append(rel)
            continue
        # Tests may sit in subdirectories, so walk rather than list.
        has_tests = any(
            f.endswith("_test.dart")
            for _, _, files in os.walk(testdir)
            for f in files
        )
        if not has_tests:
            # A test/ directory that exists but holds nothing is a different
            # fact from no test directory: the package was scaffolded for
            # tests and never given one.
            empty.append(rel)
            continue
        try:
            r = subprocess.run(["flutter", "test", "--reporter", "compact"],
                               cwd=path, capture_output=True, text=True,
                               timeout=1800)
        except subprocess.TimeoutExpired:
            broken.append((rel, "timed out"))
            continue
        out = r.stdout + r.stderr
        if "Failed to load" in out or "Error: Couldn't resolve" in out:
            broken.append((rel, "could not load test suite"))
            continue
        m = None
        for m in COUNT.finditer(out):
            pass
        if m is None:
            broken.append((rel, "no test summary produced"))
            continue
        passed = int(m.group(1))
        failed_n = int(m.group(2) or 0)
        total_pass += passed
        total_fail += failed_n
        if failed_n:
            failed.append((rel, passed, failed_n))
        else:
            clean.append((rel, passed))

    for rel, passed in clean:
        print(f"  PASS  {rel:<50} {passed} tests")
    for rel, passed, failed_n in failed:
        print(f"  FAIL  {rel:<50} {passed} passed, {failed_n} failed")
    for rel, why in broken:
        print(f"  BROKEN {rel:<49} {why}")
    for rel in empty:
        print(f"  NO TESTS {rel:<48} test/ exists but is empty")
    print()
    print(f"packages: {len(dirs)}  green: {len(clean)}  "
          f"failing: {len(failed)}  broken: {len(broken)}  "
          f"no test dir: {len(untested)}  empty test dir: {len(empty)}")
    print(f"tests passed: {total_pass}  failed: {total_fail}")
    return 0 if not failed and not broken and not empty else 1


if __name__ == "__main__":
    sys.exit(main())
