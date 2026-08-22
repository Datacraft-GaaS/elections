"""
eel-pii-in-events lint driver (HARD-24).

Walks the event-schema and service source trees, applies the Semgrep rules
defined in rules/no-tier-a-in-event.yaml, then cross-checks each finding
against allowlist.yaml.  Exits non-zero if any un-allowlisted Tier-A field
is found.

Usage:
    python -m eel_pii_in_events.lint [--root /path/to/repo]
    uv run eel-pii-lint [--root /path/to/repo]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

import yaml

# Tier-A field name patterns (raw PII, always flagged unless allowlisted).
_TIER_A_RE = re.compile(
    r"\b(national_id|voter_id|id_number|passport_number|fingerprint_raw"
    r"|iris_raw|face_image|dob_exact|full_name_dob|phone_number_raw"
    r"|email_address_raw)\b"
)

# Tier-B quasi-identifiers (flagged as warnings; also allowlistable).
_TIER_B_RE = re.compile(
    r"\b(voter_id_hashed|ward_code|constituency_code|age_bracket|county_code)\b"
)

_RULES_FILE = Path(__file__).parent / "rules" / "no-tier-a-in-event.yaml"
_ALLOWLIST_FILE = Path(__file__).parent / "allowlist.yaml"

# Scan targets relative to repo root.
_SCAN_GLOBS = [
    "packages/event-schemas/**/*",
    "services/*/crates/*/src/**/*.rs",
    "services/*/py/*/src/**/*.py",
    "services/*/ex/*/lib/**/*.ex",
]


def load_allowlist(path: Path) -> dict[str, set[str]]:
    """Return {event_type: {field, ...}} from allowlist.yaml."""
    if not path.exists():
        return {}
    data = yaml.safe_load(path.read_text())
    result: dict[str, set[str]] = {}
    for entry in data.get("allowlist", []):
        et = entry["event_type"]
        result[et] = set(entry.get("fields", []))
    return result


class Finding:
    """A single PII field occurrence."""

    def __init__(
        self,
        file: str,
        line: int,
        field: str,
        tier: str,
        event_type: str | None = None,
    ) -> None:
        self.file = file
        self.line = line
        self.field = field
        self.tier = tier
        self.event_type = event_type

    def is_allowlisted(self, allowlist: dict[str, set[str]]) -> bool:
        if self.event_type and self.event_type in allowlist:
            return self.field in allowlist[self.event_type]
        # Unknown event type: Tier-B is not blocked, Tier-A always blocked.
        return self.tier == "B"

    def __repr__(self) -> str:
        et = f" [{self.event_type}]" if self.event_type else ""
        return f"{self.file}:{self.line}: Tier-{self.tier} field '{self.field}'{et}"


def _infer_event_type(
    file_path: Path, line: int, text: str | None = None
) -> str | None:
    """Best-effort: extract the containing event type from the file path or nearby code."""
    # Prefer the filename stem (e.g. voter_registered_v1.rs -> voter_registered.v1)
    stem = file_path.stem
    if stem and not stem.startswith("mod") and not stem.startswith("lib"):
        return stem.replace("_v", ".v").replace("_", ".")
    return None


def scan_with_semgrep(root: Path, semgrep_available: bool) -> list[dict[str, Any]]:
    """Run Semgrep if available; otherwise return an empty list."""
    if not semgrep_available:
        return []
    targets = [str(root)]
    cmd = [
        "semgrep",
        "--config",
        str(_RULES_FILE),
        "--json",
        "--quiet",
        *targets,
    ]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, check=False)
        if out.returncode not in (0, 1):
            print(f"[eel-pii-lint] semgrep error: {out.stderr[:200]}", file=sys.stderr)
            return []
        return json.loads(out.stdout).get("results", [])
    except (FileNotFoundError, json.JSONDecodeError):
        return []


def grep_scan(root: Path, pattern: re.Pattern[str], tier: str) -> list[Finding]:
    """Fallback line-level scan when Semgrep is unavailable or for CI speed."""
    findings: list[Finding] = []
    globs = [
        root / "packages/event-schemas",
        *root.glob("services/*/crates/*/src"),
        *root.glob("services/*/py/*/src"),
        *root.glob("services/*/ex/*/lib"),
    ]
    for base in globs:
        if not base.exists():
            continue
        for f in base.rglob("*"):
            if not f.is_file():
                continue
            if f.suffix not in {".rs", ".py", ".ex", ".exs", ".json", ".yaml", ".yml"}:
                continue
            try:
                for lineno, text in enumerate(
                    f.read_text(errors="ignore").splitlines(), 1
                ):
                    for m in pattern.finditer(text):
                        findings.append(
                            Finding(
                                file=str(f.relative_to(root)),
                                line=lineno,
                                field=m.group(1),
                                tier=tier,
                                event_type=_infer_event_type(f, lineno, text),
                            )
                        )
            except OSError:
                continue
    return findings


def run_lint(root: Path) -> int:
    """Run the full lint pass. Returns exit code (0 = clean, 1 = violations)."""
    allowlist = load_allowlist(_ALLOWLIST_FILE)

    # Check semgrep availability.
    semgrep_ok = (
        subprocess.run(
            ["semgrep", "--version"], capture_output=True, check=False
        ).returncode
        == 0
    )

    # Always run the regex grep scan as the baseline.  Semgrep provides
    # additional structural analysis when available and succeeds.
    findings: list[Finding] = grep_scan(root, _TIER_A_RE, "A") + grep_scan(
        root, _TIER_B_RE, "B"
    )

    if semgrep_ok:
        print("[eel-pii-lint] running Semgrep rules", file=sys.stderr)
        raw = scan_with_semgrep(root, semgrep_ok)
        for r in raw:
            field = (
                r.get("extra", {})
                .get("metavars", {})
                .get("$FIELD", {})
                .get("abstract_content", "")
            )
            if not field:
                continue
            tier = r.get("extra", {}).get("metadata", {}).get("pii_tier", "A")
            file_ = r.get("path", "")
            line = r.get("start", {}).get("line", 0)
            event_type = _infer_event_type(Path(file_), line)
            findings.append(
                Finding(
                    file=file_, line=line, field=field, tier=tier, event_type=event_type
                )
            )
    else:
        print(
            "[eel-pii-lint] semgrep not found, grep scan only",
            file=sys.stderr,
        )

    violations: list[Finding] = []
    warnings: list[Finding] = []
    for finding in findings:
        if finding.is_allowlisted(allowlist):
            continue
        if finding.tier == "A":
            violations.append(finding)
        else:
            warnings.append(finding)

    for w in warnings:
        print(f"WARNING {w}", file=sys.stderr)
    for v in violations:
        print(f"ERROR   {v}")

    if violations:
        print(
            f"\n[eel-pii-lint] {len(violations)} Tier-A PII violation(s) found. "
            "Add to allowlist.yaml if intentional.",
            file=sys.stderr,
        )
        return 1

    print(
        f"[eel-pii-lint] clean ({len(warnings)} Tier-B warning(s))",
        file=sys.stderr,
    )
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description="eel PII-in-events linter (HARD-24)")
    parser.add_argument(
        "--root",
        type=Path,
        default=Path("."),
        help="Repo root (default: current directory)",
    )
    args = parser.parse_args()
    sys.exit(run_lint(args.root.resolve()))


if __name__ == "__main__":
    main()
