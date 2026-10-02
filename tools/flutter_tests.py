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

# The shell's SDKROOT on this machine points at
# `.../Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk`, which is not
# installed -- the available SDKs are MacOSX.sdk and MacOSX27. Every `flutter test`
# then dies before running a single test:
#
#   xcodebuild: error: SDK "…/MacOSX26.5.sdk" cannot be located.
#   xcrun: error: unable to find utility "lipo", not a developer tool or in PATH
#
# and the run is scored "no test summary produced", which is what 19 of the 24
# packages reported. The Dart tests are fine; the environment is stale.
#
# This is the same failure as the stale `SDKROOT` on the Rust side, and the fix is
# the same: unset it for the child process rather than trusting the shell. Doing it
# here rather than at each call site means the gate cannot regress to reporting a
# broken toolchain as 19 broken packages.
def flutter_env():
    env = dict(os.environ)
    env.pop("SDKROOT", None)
    return env


# A stray `flutter create` scaffold, committed in full, whose README still reads
# "A new Flutter project" and whose only test is the default counter widget test. It
# is not a product, and it was being measured as one -- 24 packages and 374 tests,
# of which one package and one test were this scaffold's boilerplate.
#
# Excluded rather than deleted: it is tracked, so removing it is a decision for
# whoever owns it. Same reasoning as the Cargo gate's tools/__pycache__ exclusion.
EXCLUDE_DIRS = {"test_flutter_tmp"}

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))
COUNT = re.compile(r"\+(\d+)(?:\s+-(\d+))?:\s*(?:All tests passed|Some tests failed)")


def package_dirs(targets):
    out = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames
                           if d not in (".dart_tool", "build", "node_modules",
                                        ".git", "target")]
            if "pubspec.yaml" in filenames:
                rel = os.path.relpath(dirpath, ROOT)
                if rel.split(os.sep)[0] in EXCLUDE_DIRS:
                    continue
                out.append(dirpath)
    return sorted(out)


def main():
    targets = sys.argv[1:] or ["."]
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
                               cwd=path, capture_output=True, text=True, env=flutter_env(),
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
