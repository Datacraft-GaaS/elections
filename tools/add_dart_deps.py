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

"""Add the pubspec dependencies each package imports but does not declare.

An undeclared `package:` import is an unresolvable URI in Dart, so this is the
largest single mechanical class of analyzer error in the tree.

Version constraints are not invented. Each one is copied from a package that
already uses that dependency in this repository, so the tree stays on one
version per library rather than drifting to whatever `pub add` would pick.

A package importing itself by name is legitimate and is never touched.
"""

import os
import re
import sys

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))

# Versions already in use elsewhere in this tree, chosen as the most common.
PUB_VERSIONS = {
    "crypto": "^3.0.3",
    "dio": "^5.4.0",
    "uuid": "^4.5.1",
    "local_auth": "^2.2.0",
    "flutter_secure_storage": "^9.0.0",
    "hooks_riverpod": "^2.5.0",
    "flutter_hooks": "^0.20.5",
    "isar": "^3.1.0",
    "go_router": "^13.2.0",
    "intl": "^0.20.0",
    "equatable": "^2.1.0",
    "riverpod": "^2.5.0",
    "freezed_annotation": "^2.4.1",
    "json_annotation": "^4.9.0",
    "http": "^1.2.0",
    "shared_preferences": "^2.2.0",
    "path_provider": "^2.1.0",
    "sqflite": "^2.3.0",
}

# Repo packages, added as path dependencies.
PATH_DEPS = {
    "eel_core_flutter": "packages/eel-core-flutter",
    "eel_kit_sdk": "packages/kit-sdk-flutter",
    "eel_design": "packages/eel_design",
    "eel_drivers": "packages/eel_drivers",
}

SDK_PACKAGES = {
    "flutter",
    "flutter_test",
    "flutter_localizations",
    "flutter_driver",
    "flutter_web_plugins",
    "sky_engine",
    "integration_test",
    "flutter_driver_extension",
}

IMPORT_RE = re.compile(r"""(?:import|export)\s+['\"]package:([^/'\"]+)""")
DECL_RE = re.compile(r"^  ([A-Za-z0-9_]+):", re.M)


def own_name(pubspec_path):
    with open(pubspec_path) as handle:
        match = re.search(r"^name:\s*(\S+)", handle.read(), re.M)
    return match.group(1) if match else None


def add_dependency(pubspec_path, name, version_line):
    with open(pubspec_path) as handle:
        text = handle.read()
    if re.search(rf"^\s+{re.escape(name)}:", text, re.M):
        return False
    # Append into the dependencies: block, before dev_dependencies if present.
    match = re.search(r"^dependencies:", text, re.M)
    if not match:
        return False
    insert_at = match.end()
    for dm in re.finditer(r"^dev_dependencies:", text, re.M):
        insert_at = dm.start()
        break
    block = f"\n  {name}: {version_line}\n"
    text = text[:insert_at] + block + text[insert_at:]
    with open(pubspec_path, "w") as handle:
        handle.write(text)
    return True


def main():
    targets = sys.argv[1:] or ["services", "packages"]
    package_dirs = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in (".dart_tool", "build")]
            if "pubspec.yaml" in filenames:
                package_dirs.append(dirpath)

    imported = {d: set() for d in package_dirs}
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in (".dart_tool", "build")]
            owner = next(
                (
                    d
                    for d in package_dirs
                    if dirpath == d or dirpath.startswith(d + os.sep)
                ),
                None,
            )
            if owner is None:
                continue
            for name in filenames:
                if name.endswith(".dart"):
                    with open(os.path.join(dirpath, name)) as handle:
                        imported[owner].update(IMPORT_RE.findall(handle.read()))

    added = 0
    for package_dir in sorted(package_dirs):
        pubspec = os.path.join(package_dir, "pubspec.yaml")
        with open(pubspec) as handle:
            declared = set(DECL_RE.findall(handle.read()))
        me = own_name(pubspec)
        needed = imported[package_dir] - declared - SDK_PACKAGES
        if me:
            needed.discard(me)  # a package may import itself by name
        for pkg in sorted(needed):
            if pkg in PATH_DEPS:
                rel = os.path.relpath(PATH_DEPS[pkg], package_dir)
                line = f"\n    path: {rel}"
                kind = "path"
            elif pkg in PUB_VERSIONS:
                line = PUB_VERSIONS[pkg]
                kind = "pub"
            else:
                print(
                    f"  SKIP {os.path.relpath(package_dir, ROOT)}: {pkg} (no known version)"
                )
                continue
            if add_dependency(pubspec, pkg, line):
                added += 1
                print(f"  + {os.path.relpath(package_dir, ROOT)}: {pkg} ({kind})")
    print(f"dependencies added: {added}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
