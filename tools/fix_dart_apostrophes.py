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

"""Escape apostrophes that terminate a Dart string literal early.

`const ['Voter's Register', ...]` terminates the literal at the apostrophe and
produces a cascade of parse errors from one authoring slip.

An apostrophe cannot be found by looking for a literal "containing" one: the
literal genuinely ends there, so it scans as a valid short string. The signal is
different. A closing quote immediately followed by a letter or underscore is
never valid Dart -- `'foo'bar` is a syntax error -- so such a quote is an
apostrophe the author meant to keep inside the literal.

The scanner stays *inside* the literal after flagging one, which is the part
that matters. A first attempt reset its position to just after the flagged
quote, so it lost track of being inside a string and then flagged the next
literal's *opening* quote, inserting an escape outside a string and creating a
new syntax error. Flagged quotes are treated as escapes and the walk continues
to the real terminator.

Comments, raw strings, triple-quoted strings, and string interpolation are all
skipped. Interpolation is the subtle one: in
`'Seal $id, ${ok ? 'yes' : 'no'}'` the inner quotes are separate string literals
inside `${...}`, not apostrophes. A scanner that walks into them flags a
legitimate nested literal and inserts an escape *outside* a string, which is a
new syntax error rather than a fix.
"""

import os
import sys

ROOT = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))

IDENT = set("abcdefghijklmnopqrstuvwxyz"
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            "0123456789_")


def _skip_non_literal(text, i):
    """Advance past a comment or a non-single-quoted string starting at i."""
    n = len(text)
    if text.startswith("//", i):
        j = text.find("\n", i)
        return n if j < 0 else j
    if text.startswith("/*", i):
        j = text.find("*/", i + 2)
        return n if j < 0 else j + 2
    if text.startswith('"""', i) or text.startswith("'''", i):
        quote = text[i:i + 3]
        j = text.find(quote, i + 3)
        return n if j < 0 else j + 3
    # Raw string: no escapes, so a plain scan to the closing quote is correct.
    if text[i] in "rR" and i + 1 < n and text[i + 1] in "'\"" and not text.startswith(text[i + 1] * 2, i + 1):
        quote = text[i + 1]
        j = i + 2
        while j < n and text[j] != quote:
            j += 1
        return j + 1
    if text[i] == '"':
        j = i + 1
        while j < n and text[j] != '"':
            j += 2 if text[j] == "\\" else 1
        return j + 1
    return i + 1


def _skip_interpolation(text, i):
    """Return the index just past the closing brace of a `${...}` region.

    Nested braces and nested string literals are both tracked, so an apostrophe
    inside a nested literal is not mistaken for the end of the interpolation.
    """
    n = len(text)
    depth = 1
    while i < n and depth:
        c = text[i]
        if c == "{":
            depth += 1
            i += 1
        elif c == "}":
            depth -= 1
            i += 1
        elif c in "'\"":
            quote = c
            i += 1
            while i < n:
                if text[i] == "\\":
                    i += 2
                    continue
                if text[i] == quote:
                    i += 1
                    break
                i += 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j
        else:
            i += 1
    return i


def scan(text):
    """Yield the index of every premature closing quote of a single-quoted literal."""
    out = []
    i, n = 0, len(text)
    while i < n:
        if text[i] != "'":
            i = _skip_non_literal(text, i)
            continue

        # Walk a plain single-quoted literal.
        j = i + 1
        terminated = False
        while j < n:
            c = text[j]
            if c == "\\":
                j += 2
                continue
            if c == "\n":
                break
            if c == "$" and j + 1 < n:
                if text[j + 1] == "{":
                    j = _skip_interpolation(text, j + 2)
                    continue
                if text[j + 1] in IDENT:
                    # $identifier form: skip the name.
                    j += 2
                    while j < n and (text[j].isalnum() or text[j] == "_"):
                        j += 1
                    continue
            if c == "'":
                nxt = text[j + 1] if j + 1 < n else ""
                if nxt in IDENT:
                    # An apostrophe the author meant to keep. Stay inside the
                    # literal and keep looking for the real terminator.
                    out.append(j)
                    j += 1
                    continue
                terminated = True
                break
            j += 1

        if terminated:
            i = j + 1
        else:
            i = max(j, i + 1)
    return out


def rewrite(text):
    """Escape each premature closing quote, right to left so offsets hold."""
    positions = scan(text)
    for pos in reversed(positions):
        text = text[:pos] + "\\" + text[pos:]
    return text, len(positions)


def main():
    targets = sys.argv[1:] or ["services", "packages"]
    changed = 0
    total = 0
    for base in targets:
        for dirpath, dirnames, filenames in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d not in (".dart_tool", "build")]
            for name in filenames:
                if not name.endswith(".dart"):
                    continue
                path = os.path.join(dirpath, name)
                with open(path) as handle:
                    original = handle.read()
                updated, count = rewrite(original)
                if count and updated != original:
                    with open(path, "w") as handle:
                        handle.write(updated)
                    changed += 1
                    total += count
                    print(f"  {os.path.relpath(path, ROOT)}: {count}")
    print(f"files changed: {changed}, apostrophes escaped: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
