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

"""Tests for eel_cvr_ocr.extract (OCR extraction layer)."""

from __future__ import annotations

import io

import pytest
from PIL import Image

from eel_cvr_ocr.extract import (
    TESSERACT_OK,
    _extract_death_stub,
    _extract_id_stub,
    _stub_hash,
    extract,
)
from eel_cvr_ocr.models import OcrModality


def _make_image_bytes(width: int = 200, height: int = 100) -> bytes:
    """Create a minimal valid JPEG image for testing."""
    img = Image.new("RGB", (width, height), color=(255, 255, 255))
    buf = io.BytesIO()
    img.save(buf, format="JPEG")
    return buf.getvalue()


# ---------------------------------------------------------------------------
# 1. Stub adapter: deterministic output for same bytes
# ---------------------------------------------------------------------------


def test_stub_death_is_deterministic() -> None:
    data = b"same bytes"
    f1, c1 = _extract_death_stub(data)
    f2, c2 = _extract_death_stub(data)
    assert f1 == f2
    assert c1 == c2 == 0.0


# ---------------------------------------------------------------------------
# 2. Stub adapter: different bytes -> different output
# ---------------------------------------------------------------------------


def test_stub_death_different_bytes_differs() -> None:
    f1, _ = _extract_death_stub(b"bytes_a")
    f2, _ = _extract_death_stub(b"bytes_b")
    # At least one field should differ
    assert f1 != f2


# ---------------------------------------------------------------------------
# 3. Stub adapter: ID correction is deterministic
# ---------------------------------------------------------------------------


def test_stub_id_correction_is_deterministic() -> None:
    data = b"id_bytes"
    f1, _ = _extract_id_stub(data)
    f2, _ = _extract_id_stub(data)
    assert f1 == f2


# ---------------------------------------------------------------------------
# 4. Stub adapter: confidence is always 0.0
# ---------------------------------------------------------------------------


def test_stub_confidence_zero() -> None:
    _, c = _extract_death_stub(b"any")
    assert c == 0.0


# ---------------------------------------------------------------------------
# 5. extract() returns OcrResponse with correct modality
# ---------------------------------------------------------------------------


def test_extract_death_cert_returns_correct_modality() -> None:
    image_bytes = _make_image_bytes()
    result = extract(image_bytes, OcrModality.death_certificate)
    assert result.modality == OcrModality.death_certificate


# ---------------------------------------------------------------------------
# 6. extract() returns OcrResponse with correct modality for id_correction
# ---------------------------------------------------------------------------


def test_extract_id_correction_returns_correct_modality() -> None:
    image_bytes = _make_image_bytes()
    result = extract(image_bytes, OcrModality.id_correction)
    assert result.modality == OcrModality.id_correction


# ---------------------------------------------------------------------------
# 7. extract() adapter field is 'tesseract' or 'stub'
# ---------------------------------------------------------------------------


def test_extract_adapter_field_valid() -> None:
    image_bytes = _make_image_bytes()
    result = extract(image_bytes, OcrModality.death_certificate)
    assert result.adapter in ("tesseract", "stub")


# ---------------------------------------------------------------------------
# 8. extract() stub path emits warning about deterministic placeholders
# ---------------------------------------------------------------------------


def test_extract_stub_warns_when_tesseract_absent() -> None:
    image_bytes = _make_image_bytes()
    result = extract(image_bytes, OcrModality.death_certificate)
    if not TESSERACT_OK:
        assert any("stub" in w for w in result.warnings)


# ---------------------------------------------------------------------------
# 9. Empty image bytes raises ValueError
# ---------------------------------------------------------------------------


def test_empty_image_raises() -> None:
    with pytest.raises(ValueError, match="empty"):
        extract(b"", OcrModality.death_certificate)


# ---------------------------------------------------------------------------
# 10. Invalid image bytes raises ValueError
# ---------------------------------------------------------------------------


def test_invalid_image_raises() -> None:
    with pytest.raises(ValueError, match="invalid image"):
        extract(b"not an image", OcrModality.death_certificate)


# ---------------------------------------------------------------------------
# 11. Stub hash is stable SHA-256
# ---------------------------------------------------------------------------


def test_stub_hash_is_sha256() -> None:
    import hashlib

    data = b"test_data"
    assert _stub_hash(data) == hashlib.sha256(data).hexdigest()


# ---------------------------------------------------------------------------
# 12. Stub death fields include all required keys
# ---------------------------------------------------------------------------


def test_stub_death_fields_complete() -> None:
    fields, _ = _extract_death_stub(b"complete_test")
    # All fields should be populated (not None) by the stub
    assert fields.deceased_name is not None
    assert fields.national_id is not None
    assert fields.date_of_death is not None
    assert fields.place_of_death is not None
    assert fields.cert_serial is not None
