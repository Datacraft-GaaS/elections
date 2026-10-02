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

"""Integration tests for BagBuilder and BagVerifier."""

import tempfile
from pathlib import Path

import pytest

from bagit_pkg import BagBuilder, BagVerifier


@pytest.fixture
def sample_source(tmp_path: Path) -> Path:
    """A small source directory with two text files."""
    (tmp_path / "file_a.txt").write_text("hello world", encoding="utf-8")
    (tmp_path / "file_b.txt").write_text("election data 2024", encoding="utf-8")
    return tmp_path


def test_create_bag_produces_required_files(
    sample_source: Path, tmp_path: Path
) -> None:
    dest = tmp_path / "bag_out"
    builder = BagBuilder()
    manifest = builder.create_bag(
        source_path=sample_source,
        dest_path=dest,
        metadata={"Source-Organization": "Electoral Authority", "Contact-Name": "Archive Unit"},
    )

    assert (dest / "bagit.txt").exists()
    assert (dest / "bag-info.txt").exists()
    assert (dest / "manifest-sha256.txt").exists()
    assert (dest / "data").is_dir()
    assert manifest.bagit_version == "1.0"
    assert manifest.tag_file_character_encoding == "UTF-8"
    # Two files: file_a.txt and file_b.txt
    assert len(manifest.manifest) == 2
    assert manifest.payload_oxum.endswith(".2")  # 2 streams


def test_bagit_txt_content(sample_source: Path, tmp_path: Path) -> None:
    dest = tmp_path / "bag_out"
    BagBuilder().create_bag(sample_source, dest)
    content = (dest / "bagit.txt").read_text(encoding="utf-8")
    assert "BagIt-Version: 1.0" in content
    assert "Tag-File-Character-Encoding: UTF-8" in content


def test_verify_valid_bag(sample_source: Path, tmp_path: Path) -> None:
    dest = tmp_path / "bag_out"
    BagBuilder().create_bag(sample_source, dest)
    result = BagVerifier().verify(dest)
    assert result.valid, f"Expected valid but got errors: {result.errors}"
    assert result.errors == []


def test_verify_detects_corruption(sample_source: Path, tmp_path: Path) -> None:
    dest = tmp_path / "bag_out"
    BagBuilder().create_bag(sample_source, dest)

    # Corrupt one of the data files after bagging.
    corrupted = next((dest / "data").glob("*.txt"))
    corrupted.write_text("tampered content!", encoding="utf-8")

    result = BagVerifier().verify(dest)
    assert not result.valid
    assert any("checksum mismatch" in e for e in result.errors)


def test_verify_detects_missing_file(sample_source: Path, tmp_path: Path) -> None:
    dest = tmp_path / "bag_out"
    BagBuilder().create_bag(sample_source, dest)

    # Delete a file that is listed in the manifest.
    next((dest / "data").glob("*.txt")).unlink()

    result = BagVerifier().verify(dest)
    assert not result.valid
    assert any("missing" in e for e in result.errors)


def test_verify_nonexistent_bag() -> None:
    result = BagVerifier().verify("/nonexistent/path/bag")
    assert not result.valid
    assert any("does not exist" in e for e in result.errors)


def test_bag_info_txt_contains_metadata(sample_source: Path, tmp_path: Path) -> None:
    dest = tmp_path / "bag_out"
    BagBuilder().create_bag(
        sample_source,
        dest,
        metadata={"Source-Organization": "Electoral Authority"},
    )
    info = (dest / "bag-info.txt").read_text(encoding="utf-8")
    assert "Source-Organization: the Commission" in info
    assert "Payload-Oxum:" in info
    assert "Bagging-Date:" in info
