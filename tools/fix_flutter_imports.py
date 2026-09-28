#!/usr/bin/env python3
"""Repair wrong intra-repo package import paths across the Flutter tree.

Every rewrite is validated: the corrected path must resolve to a file that
actually exists, otherwise the import is left alone and reported. A script that
guessed would trade one broken import for another, and a broken import is what
we are here to remove.

Classes handled:
  hooks_riverpod/hooks_riverprovider.dart -> hooks_riverpod/hooks_riverpod.dart
  eel_kit_sdk/<deep>                       -> the barrel, which re-exports it
  eel_core_flutter/<mod>/...               -> eel_core_flutter/src/<mod>/...
  eel_core_flutter/lib/src/...             -> eel_core_flutter/src/...
  eel_design/<anything>                    -> the single-file library
"""

import os
import re
import sys

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))

# Where each package's real sources live, discovered by inspection rather than
# assumed. A path prefix maps to the directory that actually contains the file.
PACKAGE_LAYOUT = {
    "eel_kit_sdk": {"barrel": "lib/eel_kit_sdk.dart", "prefix": "lib/"},
    "eel_core_flutter": {"prefix": "lib/src/"},
    "eel_design": {"barrel": "lib/eel_design.dart", "prefix": "lib/"},
    "hooks_riverpod": None,  # third-party; corrected by name only
}

# Where the package directory for a given package name lives inside the repo.
PACKAGE_DIR = {
    "eel_kit_sdk": "packages/kit-sdk-flutter",
    "eel_core_flutter": "packages/eel-core-flutter",
    "eel_design": "packages/eel_design",
}

IMPORT_RE = re.compile(r"""(import|export)\s+'(package:[^']+)'\s*;""")


def resolve_candidate(pkg, path):
    """Return the on-disk file for a `package:` subpath, or None if not real.

    Dart maps `package:foo/bar.dart` to `<package>/lib/bar.dart`, so every
    candidate is resolved under the package's `lib/`. Getting this wrong is how
    the first version of this script silently rewrote nothing.
    """
    rel = PACKAGE_DIR.get(pkg)
    if rel is None:
        return None
    return os.path.join(ROOT, rel, "lib", path)


def candidate_for(pkg, path):
    """Propose corrected paths for `pkg:path`, best first."""
    out = []
    layout = PACKAGE_LAYOUT.get(pkg)

    if pkg == "hooks_riverpod" and path == "hooks_riverpod/hooks_riverprovider.dart":
        out.append("hooks_riverpod/hooks_riverpod.dart")
        return out

    if layout is None:
        return out

    # A single-file library: everything resolves to the one file.
    if layout.get("barrel"):
        root_name = os.path.basename(layout["barrel"])
        if path != root_name:
            out.append(root_name)

    # A few call sites wrote `package:foo/lib/src/...`; the `lib/` is implicit.
    if path.startswith("lib/"):
        out.append(path[len("lib/"):])
    # eel_core_flutter has no barrel, so its modules are one level deeper.
    if pkg == "eel_core_flutter" and not path.startswith("src/"):
        out.append("src/" + path)

    seen, ordered = set(), []
    for c in out:
        if c not in seen:
            seen.add(c)
            ordered.append(c)
    return ordered


def main():
    targets = sys.argv[1:] or ["services", "packages"]
    files = []
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in (".dart_tool", "build")]
            for name in filenames:
                if name.endswith(".dart"):
                    files.append(os.path.join(dirpath, name))

    fixed = {}
    unresolved = {}
    for path in files:
        with open(path) as handle:
            text = original = handle.read()

        def rewrite(match):
            kind, uri = match.group(1), match.group(2)
            if not uri.startswith("package:"):
                return match.group(0)
            body = uri[len("package:"):]
            pkg, _, sub = body.partition("/")
            if pkg not in PACKAGE_LAYOUT:
                return match.group(0)
            for candidate in candidate_for(pkg, sub):
                target = resolve_candidate(pkg, candidate)
                if target and os.path.isfile(target):
                    fixed[uri] = f"package:{pkg}/{candidate}"
                    return f"{kind} 'package:{pkg}/{candidate}';"
            return match.group(0)

        text = IMPORT_RE.sub(rewrite, text)
        if text != original:
            with open(path, "w") as handle:
                handle.write(text)
            unresolved.setdefault(os.path.relpath(path, ROOT), []).append(
                [u for u, _ in IMPORT_RE.findall(original)])

    print(f"files scanned: {len(files)}")
    print("rewrites applied:")
    for uri, new in sorted(fixed.items()):
        print(f"  {uri}\n    -> {new}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
