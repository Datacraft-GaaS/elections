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
import pathlib
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


# An interaction handler with an empty body: `onPressed: () {}`.
#
# `flutter analyze` cannot see this -- an empty closure is valid Dart, and
# `EelButton` takes a nullable `VoidCallback`, so passing one type-checks and
# renders a live-looking control that does nothing when tapped. 34 of these sat
# across 9 apps, and neither the analyzer nor the completeness rule caught them:
# `tools/completeness/rules/dart_stub_ui.dart` looks for `Text('...stub')` string
# literals, not empty callbacks.
#
# A control that cannot do the thing must not look like it can. `EelButton` already
# treats `onPressed: null` as disabled, so the honest form needs no new API.
EMPTY_CALLBACK = re.compile(r"\b(?:onPressed|onTap|onLongPress)\s*:\s*\(\s*\)\s*\{\s*\}")

# Widgets under test legitimately pass an empty callback: a golden test is checking
# how a button *renders*, and a no-op is the point.
CALLBACK_EXEMPT = ("/test/", "/fixtures/", "design-system/flutter/test")


def empty_callbacks(path):
    """(line number, line) for every empty interaction handler in a Dart file."""
    try:
        text = pathlib.Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    if any(marker in path.replace(os.sep, "/") for marker in CALLBACK_EXEMPT):
        return []
    return [(n, ln.strip()) for n, ln in enumerate(text.splitlines(), 1)
            if EMPTY_CALLBACK.search(ln)]


def stub_callbacks(dirs):
    """Every empty interaction handler across a set of package lib trees."""
    found = []
    for d in dirs:
        lib = os.path.join(d, "lib")
        if not os.path.isdir(lib):
            continue
        for dirpath, dirnames, filenames in os.walk(lib):
            dirnames[:] = [x for x in dirnames if x not in (".dart_tool", "build")]
            for fn in filenames:
                if fn.endswith(".dart"):
                    for lineno, line in empty_callbacks(os.path.join(dirpath, fn)):
                        found.append((os.path.relpath(os.path.join(dirpath, fn), ROOT),
                                      lineno, line))
    return found


ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))
ERROR_RE = re.compile(r"^\s*error •", re.M)
WARN_RE = re.compile(r"^\s*warning •", re.M)
INFO_RE = re.compile(r"^\s*info •", re.M)
# "1 issue found" is singular, so matching the plural alone hides any
# package with exactly one finding.
SUMMARY_RE = re.compile(r"\bissues? found", re.I)


# A stray `flutter create` scaffold, committed in full -- pubspec, Android
# manifest, launcher icons at five densities, and the default counter widget test.
# Its README still reads "A new Flutter project". It is not a product, and it was
# being measured as one: the gates reported 24 packages and 374 tests, of which one
# package and one test were this scaffold's boilerplate.
#
# Excluded rather than deleted. It is tracked, so removing it is a decision for
# whoever owns it; but a count that includes it is not a count of anything, which is
# the same reason the Cargo gate excludes tools/__pycache__.
EXCLUDE_DIRS = {"test_flutter_tmp"}

SKIP = {".dart_tool", "build", "node_modules", ".git", "target"}


def package_dirs(targets):
    found = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in SKIP]
            if "pubspec.yaml" in filenames:
                rel = os.path.relpath(dirpath, ROOT)
                if rel.split(os.sep)[0] in EXCLUDE_DIRS:
                    continue
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
                             capture_output=True, text=True, timeout=600,
                             env=flutter_env())
        if pub.returncode != 0:
            tail = [l for l in pub.stderr.splitlines() if l.strip()][-1:] or ["(no detail)"]
            unmeasurable.append((rel, tail[0][:100]))
            rows.append((rel, None, None, None))
            continue

        try:
            result = subprocess.run(["flutter", "analyze"], cwd=path,
                                    capture_output=True, text=True, timeout=900,
                                    env=flutter_env())
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

    stubs = stub_callbacks(dirs)
    if stubs:
        print(f"empty interaction handlers (a control that looks live and does nothing): {len(stubs)}")
        print("-" * 78)
        for path, lineno, line in stubs:
            print(f"  {path}:{lineno}")
            print(f"      {line[:90]}")
        print()

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
