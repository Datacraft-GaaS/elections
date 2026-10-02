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

"""Tests for eel-pii-in-events lint driver (HARD-24). 12+ cases."""

from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from eel_pii_in_events.lint import (
    Finding,
    _TIER_A_RE,
    _TIER_B_RE,
    grep_scan,
    load_allowlist,
    run_lint,
)


# ---- load_allowlist ----


def test_load_allowlist_parses_entries(tmp_path: Path) -> None:
    al = tmp_path / "allowlist.yaml"
    al.write_text(
        "allowlist:\n  - event_type: foo.bar.v1\n    fields: [voter_id_hashed]\n"
    )
    result = load_allowlist(al)
    assert "foo.bar.v1" in result
    assert "voter_id_hashed" in result["foo.bar.v1"]


def test_load_allowlist_missing_file_returns_empty(tmp_path: Path) -> None:
    result = load_allowlist(tmp_path / "nonexistent.yaml")
    assert result == {}


def test_load_allowlist_empty_allowlist_key(tmp_path: Path) -> None:
    al = tmp_path / "allowlist.yaml"
    al.write_text("allowlist: []\n")
    result = load_allowlist(al)
    assert result == {}


# ---- Finding.is_allowlisted ----


def test_tier_a_not_allowlisted_when_no_allowlist_entry() -> None:
    f = Finding(
        file="foo.rs", line=1, field="voter_id", tier="A", event_type="foo.bar.v1"
    )
    assert not f.is_allowlisted({})


def test_tier_a_allowlisted_when_field_in_allowlist() -> None:
    f = Finding(
        file="foo.rs",
        line=1,
        field="voter_id_hashed",
        tier="A",
        event_type="foo.bar.v1",
    )
    allowlist = {"foo.bar.v1": {"voter_id_hashed"}}
    assert f.is_allowlisted(allowlist)


def test_tier_b_is_allowlisted_by_default_unknown_event() -> None:
    f = Finding(file="foo.rs", line=1, field="county_code", tier="B", event_type=None)
    assert f.is_allowlisted({})


def test_tier_a_not_allowlisted_wrong_event_type() -> None:
    f = Finding(
        file="foo.rs", line=1, field="voter_id", tier="A", event_type="other.event.v1"
    )
    allowlist = {"foo.bar.v1": {"voter_id"}}
    assert not f.is_allowlisted(allowlist)


# ---- grep_scan: Tier-A flagged ----


def _make_repo(tmp_path: Path, subpath: str, content: str) -> Path:
    full = tmp_path / subpath
    full.parent.mkdir(parents=True, exist_ok=True)
    full.write_text(content)
    return tmp_path


def test_grep_scan_flags_national_id_in_rust(tmp_path: Path) -> None:
    root = _make_repo(
        tmp_path,
        "packages/event-schemas/foo.rs",
        "pub national_id: String,\n",
    )
    findings = grep_scan(root, _TIER_A_RE, "A")
    assert any(f.field == "national_id" for f in findings)


def test_grep_scan_flags_voter_id_in_python(tmp_path: Path) -> None:
    root = _make_repo(
        tmp_path,
        "packages/event-schemas/event.py",
        "voter_id: str\n",
    )
    findings = grep_scan(root, _TIER_A_RE, "A")
    assert any(f.field == "voter_id" for f in findings)


def test_grep_scan_no_findings_on_clean_file(tmp_path: Path) -> None:
    root = _make_repo(
        tmp_path,
        "packages/event-schemas/clean.rs",
        "pub station_code: String,\n",
    )
    findings = grep_scan(root, _TIER_A_RE, "A")
    assert findings == []


# ---- allowlisted field passes ----


def test_allowlisted_field_not_in_violations(tmp_path: Path) -> None:
    # voter_id_hashed is Tier-B (defaulted to OK for unknown event types).
    root = _make_repo(
        tmp_path,
        "packages/event-schemas/reg.rs",
        "pub voter_id_hashed: String,\n",
    )
    findings = grep_scan(root, _TIER_B_RE, "B")
    for f in findings:
        assert f.is_allowlisted({})  # Tier-B with no event_type is auto-allowed


# ---- run_lint integration ----


def test_run_lint_clean_repo_returns_zero(tmp_path: Path) -> None:
    # Create a minimal repo with no PII fields.
    schemas = tmp_path / "packages" / "event-schemas"
    schemas.mkdir(parents=True)
    (schemas / "good.rs").write_text("pub station_code: String,\n")
    code = run_lint(tmp_path)
    assert code == 0


def test_run_lint_tier_a_violation_returns_one(tmp_path: Path) -> None:
    schemas = tmp_path / "packages" / "event-schemas"
    schemas.mkdir(parents=True)
    (schemas / "bad.rs").write_text("pub voter_id: String,\n")
    code = run_lint(tmp_path)
    assert code == 1


def test_run_lint_allowlisted_tier_b_returns_zero(tmp_path: Path) -> None:
    schemas = tmp_path / "packages" / "event-schemas"
    schemas.mkdir(parents=True)
    (schemas / "reg.rs").write_text("pub voter_id_hashed: String,\n")
    code = run_lint(tmp_path)
    assert code == 0


def test_run_lint_unknown_fields_treated_as_tier_a(tmp_path: Path) -> None:
    """Unknown class of field matching Tier-A pattern is a violation."""
    schemas = tmp_path / "packages" / "event-schemas"
    schemas.mkdir(parents=True)
    (schemas / "bad.py").write_text("passport_number: str\n")
    code = run_lint(tmp_path)
    assert code == 1
