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

"""Report package imports that the importing package does not declare.

Dart will not resolve `package:foo/...` unless `foo` is a dependency or a
dev_dependency, so an undeclared import is an unresolvable URI. Across this tree
that accounted for a large share of the analyzer errors, and it is entirely
mechanical to detect.

SDK packages (flutter, flutter_test, flutter_driver, sky_engine, integration_test)
are always available and are not reported.
"""

import os
import re
import sys

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))

SDK_PACKAGES = {
    "flutter", "flutter_test", "flutter_localizations", "flutter_driver",
    "flutter_web_plugins", "sky_engine", "integration_test", "flutter_driver_extension",
}

IMPORT_RE = re.compile(r"""(?:import|export)\s+['\"]package:([^/'\"]+)""")


def declared(pubspec_path):
    """Package names declared in dependencies or dev_dependencies."""
    names = set()
    if not os.path.isfile(pubspec_path):
        return names
    with open(pubspec_path) as handle:
        for line in handle:
            match = re.match(r"^  ([A-Za-z0-9_]+):", line)
            if match:
                names.add(match.group(1))
    return names


def main():
    targets = sys.argv[1:] or ["services", "packages"]

    # First pass: every package directory, found by walking for a pubspec.
    package_dirs = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in (".dart_tool", "build")]
            if "pubspec.yaml" in filenames:
                package_dirs.append(dirpath)

    # Second pass: the packages each directory imports. This has to be a
    # separate walk over the whole subtree, because os.walk reports only the
    # current directory's filenames and the Dart sources live under lib/ --
    # scanning filenames during the pubspec walk finds nothing at all.
    imports = {d: set() for d in package_dirs}
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in (".dart_tool", "build")]
            owner = next((d for d in package_dirs if dirpath.startswith(d + os.sep) or dirpath == d), None)
            if owner is None:
                continue
            for name in filenames:
                if not name.endswith(".dart"):
                    continue
                with open(os.path.join(dirpath, name)) as handle:
                    for pkg in IMPORT_RE.findall(handle.read()):
                        imports[owner].add(pkg)

    rows = []
    for package_dir in package_dirs:
        declared_names = declared(os.path.join(package_dir, "pubspec.yaml"))
        missing = sorted(imports[package_dir] - declared_names - SDK_PACKAGES)
        if missing:
            rows.append((os.path.relpath(package_dir, ROOT), missing))

    for rel, missing in sorted(rows):
        print(f"{rel}\n    missing: {', '.join(missing)}")
    print(f"\npackages with undeclared imports: {len(rows)} of {len(package_dirs)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
