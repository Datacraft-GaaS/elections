#!/usr/bin/env python3
"""Parse-check Rust sources without compiling them.

This machine cannot link: the Xcode command line tools have not had their
licence accepted, so `cargo check` and `cargo test` fail for every crate in the
workspace with `linking with 'cc' failed`. That makes "just compile it" an
unavailable verification step, which is a trap: an unverified Rust edit looks
exactly like a verified one until someone tries to build it.

`rustfmt` parses Rust without linking, so it catches syntax errors on a machine
where nothing compiles. That is a weaker guarantee than a type check -- it says
nothing about names, traits or types -- but it is far stronger than nothing, and
it is the strongest check available here.

It caught a real one: a handler rewritten to return `ApiError` had picked up a
doubled closing paren, `State<Arc<AppState>>))`, which a raw paren count scored
as balanced and which only a parser rejected.

Exit status is nonzero if any file fails to parse, so this can gate a commit.

Usage:
    python3 tools/rustfmt_parse_check.py                  # all changed .rs files
    python3 tools/rustfmt_parse_check.py --staged         # staged only
    python3 tools/rustfmt_parse_check.py path/to/a.rs ...
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def parse_errors(path: Path) -> list[str]:
    """rustfmt parse errors for one file. Formatting diffs are not errors."""
    try:
        proc = subprocess.run(
            ["rustfmt", "--edition", "2021", "--check", str(path)],
            capture_output=True, text=True, timeout=120,
        )
    except FileNotFoundError:
        return ["rustfmt not installed"]
    except subprocess.TimeoutExpired:
        return ["rustfmt timed out"]
    out = proc.stdout + proc.stderr
    return [ln for ln in out.splitlines() if ln.startswith("error")]


def changed_files() -> list[Path]:
    """Rust files this branch touched relative to HEAD, plus the staged set."""
    out: set[Path] = set()
    for args in (["diff", "--name-only", "HEAD"], ["diff", "--name-only", "--cached"]):
        proc = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
        for line in proc.stdout.splitlines():
            if line.endswith(".rs"):
                out.add(ROOT / line)
    return sorted(out)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="*", help="explicit files to check")
    ap.add_argument("--staged", action="store_true", help="only staged files")
    args = ap.parse_args()

    if args.paths:
        files = [Path(p) for p in args.paths]
    elif args.staged:
        proc = subprocess.run(["git", "diff", "--name-only", "--cached"],
                              cwd=ROOT, capture_output=True, text=True)
        files = [ROOT / ln for ln in proc.stdout.splitlines() if ln.endswith(".rs")]
    else:
        files = changed_files()

    if not files:
        print("no Rust files changed")
        return 0

    bad = 0
    for f in files:
        if not f.exists():
            continue
        errs = parse_errors(f)
        rel = f.relative_to(ROOT) if f.is_relative_to(ROOT) else f
        if errs:
            bad += 1
            print(f"PARSE FAIL {rel}")
            for e in errs[:4]:
                print(f"    {e}")
        else:
            print(f"ok          {rel}")

    print(f"\n{len(files) - bad}/{len(files)} parse clean")
    if bad:
        print("NOTE: this is a syntax check only, not a type check. Nothing on "
              "this machine can link until the Xcode licence is accepted.")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
