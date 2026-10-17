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

"""Check that every Cargo manifest in the repo can be loaded.

`cargo metadata` resolves the manifest and the workspace graph without
compiling anything, so this stays cheap: it creates no build output. That
matters on a full disk, and it means the check can run on every change.

A manifest that cannot be loaded is a broken workspace, not a failed build:
cargo refuses to run at all, including `cargo clean`. Three manifests in this
repo are deliberately unloadable and are listed in the root workspace's
`exclude`, so the root exclude list is honoured rather than reporting them.

Exits non-zero if any manifest outside the exclude list fails to load.
"""

import os
import subprocess
import sys

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    tomllib = None

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))
SKIP_DIRS = {".git", "node_modules", "target", "__pycache__", ".dart_tool"}


def excluded_paths():
    """Paths the root workspace explicitly excludes, as plain directories.

    Parsed with tomllib rather than a regex. The exclude list is full of
    explanatory comments containing commas, so splitting it on commas produced
    entries like "# softhsm, spectral: placeholder-only tools" and silently
    failed to exclude the paths it named.
    """
    manifest = os.path.join(ROOT, "Cargo.toml")
    if not os.path.exists(manifest) or tomllib is None:
        return set()
    with open(manifest, "rb") as fh:
        data = tomllib.load(fh)
    return {
        os.path.normpath(os.path.join(ROOT, p))
        for p in data.get("workspace", {}).get("exclude", [])
    }


def owns_workspace(manifest):
    """True when this manifest declares its own [workspace], i.e. is standalone."""
    with open(manifest, "rb") as fh:
        try:
            return "workspace" in tomllib.load(fh)
        except tomllib.TOMLDecodeError:
            return False


def owner(manifest):
    """The nearest ancestor manifest declaring a [workspace], or None."""
    d = os.path.dirname(manifest)
    while d.startswith(ROOT):
        candidate = os.path.join(d, "Cargo.toml")
        if os.path.exists(candidate) and candidate != manifest:
            if owns_workspace(candidate):
                return candidate
        d = os.path.dirname(d)
    return None


def manifests():
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        if "Cargo.toml" in filenames:
            yield os.path.join(dirpath, "Cargo.toml")


def main():
    skip = excluded_paths()
    checked = skipped = 0
    broken = []

    for manifest in sorted(manifests()):
        directory = os.path.dirname(manifest)
        if directory in skip:
            skipped += 1
            continue
        # A manifest inside a workspace cannot be loaded standalone: it
        # inherits from the workspace and cargo rejects the probe even though
        # the workspace itself is fine. So resolve it through the workspace that
        # owns it, and only probe standalone manifests directly.
        if not owns_workspace(manifest):
            workspace = owner(manifest)
            if workspace is None:
                r = subprocess.run(
                    ["cargo", "metadata", "--no-deps", "--format-version", "1",
                     "--manifest-path", manifest],
                    capture_output=True, text=True, timeout=300)
                checked += 1
                if r.returncode != 0:
                    first = next((ln for ln in r.stderr.splitlines()
                                  if ln.startswith("error")),
                                 r.stderr.strip()[:100])
                    broken.append((os.path.relpath(directory, ROOT), first))
                continue
            manifest = workspace
        r = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1",
             "--manifest-path", manifest],
            capture_output=True, text=True, timeout=300)
        checked += 1
        if r.returncode != 0:
            first = next((ln for ln in r.stderr.splitlines()
                          if ln.startswith("error")), r.stderr.strip()[:100])
            broken.append((os.path.relpath(directory, ROOT), first))

    for rel, why in broken:
        print(f"  BROKEN {rel}: {why}")
    print()
    print(f"manifests checked: {checked}  excluded: {skipped} "
          f"(of {len(skip)} exclude entries)  broken: {len(broken)}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
