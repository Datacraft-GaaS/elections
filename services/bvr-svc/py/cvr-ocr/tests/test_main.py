"""Tests for eel_cvr_ocr.main (FastAPI application layer)."""

from __future__ import annotations

import io

import pytest
from fastapi.testclient import TestClient
from PIL import Image

from eel_cvr_ocr.main import app

client = TestClient(app)


def _make_jpeg_bytes(width: int = 200, height: int = 100) -> bytes:
    img = Image.new("RGB", (width, height), color=(240, 240, 240))
    buf = io.BytesIO()
    img.save(buf, format="JPEG")
    return buf.getvalue()


# ---------------------------------------------------------------------------
# 1. Health check returns 200 + expected keys
# ---------------------------------------------------------------------------


def test_health_returns_ok() -> None:
    r = client.get("/health")
    assert r.status_code == 200
    body = r.json()
    assert body["status"] == "ok"
    assert "tesseract_available" in body
    assert "version" in body


# ---------------------------------------------------------------------------
# 2. POST /ocr/death-certificate with valid JPEG returns 200
# ---------------------------------------------------------------------------


def test_death_cert_valid_jpeg() -> None:
    data = _make_jpeg_bytes()
    r = client.post(
        "/ocr/death-certificate",
        files={"file": ("cert.jpg", data, "image/jpeg")},
    )
    assert r.status_code == 200
    body = r.json()
    assert body["modality"] == "death_certificate"
    assert "confidence" in body
    assert "extracted" in body


# ---------------------------------------------------------------------------
# 3. POST /ocr/id-correction with valid JPEG returns 200
# ---------------------------------------------------------------------------


def test_id_correction_valid_jpeg() -> None:
    data = _make_jpeg_bytes()
    r = client.post(
        "/ocr/id-correction",
        files={"file": ("form.jpg", data, "image/jpeg")},
    )
    assert r.status_code == 200
    body = r.json()
    assert body["modality"] == "id_correction"


# ---------------------------------------------------------------------------
# 4. POST /ocr/death-certificate with empty file returns 422
# ---------------------------------------------------------------------------


def test_death_cert_empty_file_rejected() -> None:
    r = client.post(
        "/ocr/death-certificate",
        files={"file": ("empty.jpg", b"", "image/jpeg")},
    )
    assert r.status_code == 422


# ---------------------------------------------------------------------------
# 5. POST /ocr/id-correction with non-image bytes returns 422
# ---------------------------------------------------------------------------


def test_id_correction_invalid_bytes_rejected() -> None:
    r = client.post(
        "/ocr/id-correction",
        files={"file": ("bad.jpg", b"not an image", "image/jpeg")},
    )
    assert r.status_code == 422


# ---------------------------------------------------------------------------
# 6. Response confidence is in [0, 1]
# ---------------------------------------------------------------------------


def test_confidence_in_valid_range() -> None:
    data = _make_jpeg_bytes()
    r = client.post(
        "/ocr/death-certificate",
        files={"file": ("cert.jpg", data, "image/jpeg")},
    )
    assert r.status_code == 200
    confidence = r.json()["confidence"]
    assert 0.0 <= confidence <= 1.0


# ---------------------------------------------------------------------------
# 7. Response adapter field is either 'tesseract' or 'stub'
# ---------------------------------------------------------------------------


def test_adapter_field_valid_value() -> None:
    data = _make_jpeg_bytes()
    r = client.post(
        "/ocr/id-correction",
        files={"file": ("form.jpg", data, "image/jpeg")},
    )
    assert r.status_code == 200
    assert r.json()["adapter"] in ("tesseract", "stub")


# ---------------------------------------------------------------------------
# 8. PNG image also accepted (not just JPEG)
# ---------------------------------------------------------------------------


def test_png_image_accepted() -> None:
    img = Image.new("RGB", (200, 100), color=(200, 200, 200))
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    data = buf.getvalue()
    r = client.post(
        "/ocr/death-certificate",
        files={"file": ("cert.png", data, "image/png")},
    )
    assert r.status_code == 200


# ---------------------------------------------------------------------------
# 9. Extracted fields match model schema (no extra keys)
# ---------------------------------------------------------------------------


def test_death_cert_extracted_fields_no_extra() -> None:
    data = _make_jpeg_bytes()
    r = client.post(
        "/ocr/death-certificate",
        files={"file": ("cert.jpg", data, "image/jpeg")},
    )
    assert r.status_code == 200
    extracted = r.json()["extracted"]
    allowed = {
        "deceased_name",
        "national_id",
        "date_of_death",
        "place_of_death",
        "cert_serial",
    }
    assert set(extracted.keys()) <= allowed


# ---------------------------------------------------------------------------
# 10. Warnings list present in response
# ---------------------------------------------------------------------------


def test_warnings_list_present() -> None:
    data = _make_jpeg_bytes()
    r = client.post(
        "/ocr/death-certificate",
        files={"file": ("cert.jpg", data, "image/jpeg")},
    )
    assert r.status_code == 200
    assert isinstance(r.json()["warnings"], list)
