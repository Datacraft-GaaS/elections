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

"""tests/test_validate.py -- pytest suite for validate.py"""

import json
import sys
from pathlib import Path

import pytest

# Add parent to path so we can import validate directly.
sys.path.insert(0, str(Path(__file__).parent.parent))

from validate import validate_manifest, cross_check

VALID_MANIFEST = {
    "schema_version": "1",
    "bundle_id": "test-bundle",
    "created_at": "2026-01-01T00:00:00Z",
    "tally_root": "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    "artefacts": [
        {
            "name": "eel-test-image",
            "kind": "image",
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "source_ref": "0000000000000000000000000000000000000001",
            "build_recipe": "packages.x86_64-linux.eel-test-image",
        }
    ],
}


def test_valid_manifest_passes():
    errors = validate_manifest(VALID_MANIFEST)
    assert errors == [], f"Expected no errors, got: {errors}"


def test_missing_sha256_rejected():
    m = json.loads(json.dumps(VALID_MANIFEST))
    del m["artefacts"][0]["sha256"]
    errors = validate_manifest(m)
    assert errors, "Expected validation error for missing sha256"


def test_unknown_schema_version_rejected():
    m = json.loads(json.dumps(VALID_MANIFEST))
    m["schema_version"] = "99"
    errors = validate_manifest(m)
    assert any(
        "schema_version" in e.lower() or "unsupported" in e.lower() for e in errors
    ), f"Expected schema_version error, got: {errors}"


def test_cross_check_finds_mismatches():
    index = {
        "contents": {
            "eel-test-image": "deadbeef" + "0" * 56,
        }
    }
    errors = cross_check(VALID_MANIFEST, index)
    assert errors, "Expected cross-check mismatch"
    assert any("eel-test-image" in e for e in errors)


def test_cross_check_passes_on_match():
    declared_sha = VALID_MANIFEST["artefacts"][0]["sha256"]
    index = {
        "contents": {
            "eel-test-image": declared_sha,
        }
    }
    errors = cross_check(VALID_MANIFEST, index)
    assert errors == [], f"Expected no errors on matching index, got: {errors}"


def test_invalid_sha256_pattern_rejected():
    m = json.loads(json.dumps(VALID_MANIFEST))
    m["artefacts"][0]["sha256"] = "not-a-hash"
    errors = validate_manifest(m)
    assert errors, "Expected error for invalid sha256 pattern"


def test_invalid_kind_rejected():
    m = json.loads(json.dumps(VALID_MANIFEST))
    m["artefacts"][0]["kind"] = "widget"
    errors = validate_manifest(m)
    assert errors, "Expected error for unknown kind"


def test_cross_check_missing_from_index():
    index = {"contents": {}}  # Empty index, name not present.
    errors = cross_check(VALID_MANIFEST, index)
    assert any("not found" in e for e in errors)
