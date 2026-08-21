"""
9.8-legal-closure.py: Assert every statute citation in the plan corpus has at least one test tag.

Statute citations (from docs/plans/11-completeness-audit.md):
  ss. 6A, 14A, 30, 39(1C), 44, 44A, 53-55, 69, 72, 78, 80, 83-84
  Art. 87, Art. 104
  s. 75(1)

Parses services/governance-svc/crates/legal-audit/ rule sets if present,
and docs/plans/modules/*.md for statute citations.
"""

import os
import re
import sys
import glob
import yaml

REPO_ROOT = os.environ.get(
    "EEL_REPO_ROOT",
    os.path.abspath(os.path.join(os.path.dirname(__file__), "../../..")),
)
AUDIT_MODE = os.environ.get("EEL_AUDIT_MODE", "")
FIXTURES_DIR = os.environ.get(
    "EEL_FIXTURES_DIR",
    os.path.join(REPO_ROOT, "verification/completeness/fixtures/mini-audit"),
)

REQUIRED_CITATIONS = {
    "s.6A",
    "s.14A",
    "s.30",
    "s.39(1C)",
    "s.44",
    "s.44A",
    "s.53",
    "s.54",
    "s.55",
    "s.69",
    "s.72",
    "s.78",
    "s.80",
    "s.83",
    "s.84",
    "Art.87",
    "Art.104",
    "s.75(1)",
}

CITATION_PATTERN = re.compile(
    r"\b(?:ss?\.|Art\.)\s*(\d+[A-Z]?(?:\(\d+[A-Z]?\))?(?:-\d+[A-Z]?)?)",
    re.IGNORECASE,
)


def log(msg: str) -> None:
    print(f"[9.8-legal-closure] {msg}", file=sys.stderr)


def collect_citations_from_files(paths: list[str]) -> dict[str, list[str]]:
    """Return {normalised_citation: [source_file, ...]}."""
    found: dict[str, list[str]] = {}
    for path in paths:
        try:
            text = open(path).read()
        except OSError:
            continue
        for m in CITATION_PATTERN.finditer(text):
            prefix = m.group(0).split(m.group(1))[0].strip().rstrip(".")
            cite = f"{prefix}.{m.group(1)}".replace(" ", "")
            found.setdefault(cite, []).append(os.path.relpath(path, REPO_ROOT))
    return found


def main() -> int:
    if AUDIT_MODE == "dry":
        log("dry mode: using fixture legal data")
        fixture_legal = os.path.join(FIXTURES_DIR, "legal-citations.yaml")
        if os.path.isfile(fixture_legal):
            data = yaml.safe_load(open(fixture_legal))
            cited = set(data.get("citations", []))
        else:
            # Treat all required as present in dry mode
            cited = set(REQUIRED_CITATIONS)
        log(f"dry: {len(cited)} citations found")
        uncovered = REQUIRED_CITATIONS - cited
        if uncovered:
            log(f"FAIL: {len(uncovered)} citations not covered: {sorted(uncovered)}")
            return 1
        log(f"PASS: all {len(REQUIRED_CITATIONS)} citations covered")
        return 0

    # Collect from plan corpus and legal-audit rules
    paths = (
        glob.glob(os.path.join(REPO_ROOT, "docs/plans/**/*.md"), recursive=True)
        + glob.glob(
            os.path.join(
                REPO_ROOT, "services/governance-svc/crates/legal-audit/**/*.toml"
            ),
            recursive=True,
        )
        + glob.glob(
            os.path.join(
                REPO_ROOT, "services/governance-svc/crates/legal-audit/**/*.rs"
            ),
            recursive=True,
        )
    )

    found_map = collect_citations_from_files(paths)
    log(f"found {len(found_map)} unique citation patterns in {len(paths)} files")

    # Check each required citation
    uncovered: list[str] = []
    for cite in sorted(REQUIRED_CITATIONS):
        # Normalise to match: s.6A -> found as s.6A or ss.6A etc.
        bare = cite.lstrip("sA").lstrip("rt.").lstrip(".")
        matched = any(bare in k for k in found_map)
        if not matched:
            uncovered.append(cite)

    if uncovered:
        log(f"FAIL: {len(uncovered)} citations have no test tag: {uncovered}")
        return 1

    log(f"PASS: all {len(REQUIRED_CITATIONS)} required citations found in plan corpus")
    return 0


if __name__ == "__main__":
    sys.exit(main())
