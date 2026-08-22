"""OCR extraction adapters for CVR field-officer certificates.

Two adapters are provided:

1. TesseractAdapter: uses OpenCV for pre-processing (thresholding, deskew,
   noise removal) and pytesseract for text extraction. Active when tesseract-ocr
   binary is present on PATH.

2. StubAdapter: deterministic hash-derived extraction used when tesseract is
   absent (e.g. CI, unit tests, devices without tesseract). Produces stable,
   reproducible output so tests can assert on field values.

The top-level extract() function selects the adapter automatically.
"""

from __future__ import annotations

import hashlib
import importlib.util
import logging
import re
from typing import Any

import numpy as np
from PIL import Image

from eel_cvr_ocr.models import (
    DeathCertificateFields,
    IdCorrectionFields,
    OcrModality,
    OcrResponse,
)

_LOG = logging.getLogger("eel_cvr_ocr.extract")

# ---------------------------------------------------------------------------
# Tesseract availability check
# ---------------------------------------------------------------------------


def _tesseract_available() -> bool:
    if importlib.util.find_spec("pytesseract") is None:
        return False
    try:
        import pytesseract  # type: ignore[import-untyped]

        pytesseract.get_tesseract_version()
        return True
    except Exception:
        return False


TESSERACT_OK: bool = _tesseract_available()

if not TESSERACT_OK:
    _LOG.warning(
        "WARN: pytesseract / tesseract-ocr not available; "
        "falling back to deterministic stub adapter. "
        "Install tesseract-ocr and pip install pytesseract[ocr] for real OCR."
    )

# ---------------------------------------------------------------------------
# Image pre-processing (shared)
# ---------------------------------------------------------------------------


def _preprocess(image: Image.Image) -> np.ndarray[Any, np.dtype[np.uint8]]:
    """Convert PIL image to a binarised, deskewed numpy array for OCR."""
    import cv2  # type: ignore[import-untyped]

    arr = np.array(image.convert("L"))  # grayscale
    # Adaptive threshold to handle uneven illumination from field photos.
    thresh = cv2.adaptiveThreshold(
        arr,
        255,
        cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
        cv2.THRESH_BINARY,
        blockSize=31,
        C=10,
    )
    # Mild denoising
    denoised: np.ndarray[Any, np.dtype[np.uint8]] = cv2.fastNlMeansDenoising(
        thresh, h=10
    )
    return denoised


# ---------------------------------------------------------------------------
# Tesseract adapter
# ---------------------------------------------------------------------------


def _date_pattern(text: str) -> str | None:
    m = re.search(r"\b(\d{1,2}[/-]\d{1,2}[/-]\d{2,4}|\d{4}-\d{2}-\d{2})\b", text)
    return m.group(1) if m else None


def _id_pattern(text: str) -> str | None:
    m = re.search(r"\b(\d{7,10})\b", text)
    return m.group(1) if m else None


def _extract_death_tesseract(
    image: Image.Image,
) -> tuple[DeathCertificateFields, float]:
    import pytesseract  # type: ignore[import-untyped]

    preprocessed = _preprocess(image)
    pil_proc = Image.fromarray(preprocessed)
    text: str = pytesseract.image_to_string(pil_proc, config="--psm 6")
    # Confidence via image_to_data
    data = pytesseract.image_to_data(pil_proc, output_type=pytesseract.Output.DICT)
    confs = [int(c) for c in data["conf"] if str(c) != "-1" and int(c) >= 0]
    confidence = (sum(confs) / len(confs) / 100.0) if confs else 0.0

    lines = text.splitlines()
    name: str | None = None
    for line in lines:
        if re.search(r"(full name|name of deceased|deceased)", line, re.I):
            parts = re.split(r"[:：]", line, maxsplit=1)
            if len(parts) > 1:
                name = parts[1].strip() or None

    fields = DeathCertificateFields(
        deceased_name=name,
        national_id=_id_pattern(text),
        date_of_death=_date_pattern(text),
        place_of_death=None,
        cert_serial=None,
    )
    return fields, confidence


def _extract_id_correction_tesseract(
    image: Image.Image,
) -> tuple[IdCorrectionFields, float]:
    import pytesseract  # type: ignore[import-untyped]

    preprocessed = _preprocess(image)
    pil_proc = Image.fromarray(preprocessed)
    text: str = pytesseract.image_to_string(pil_proc, config="--psm 6")
    data = pytesseract.image_to_data(pil_proc, output_type=pytesseract.Output.DICT)
    confs = [int(c) for c in data["conf"] if str(c) != "-1" and int(c) >= 0]
    confidence = (sum(confs) / len(confs) / 100.0) if confs else 0.0

    fields = IdCorrectionFields(
        national_id=_id_pattern(text),
        date_of_birth=_date_pattern(text),
    )
    return fields, confidence


# ---------------------------------------------------------------------------
# Stub adapter (deterministic, hash-derived)
# ---------------------------------------------------------------------------


def _stub_hash(image_bytes: bytes) -> str:
    return hashlib.sha256(image_bytes).hexdigest()


def _extract_death_stub(image_bytes: bytes) -> tuple[DeathCertificateFields, float]:
    h = _stub_hash(image_bytes)
    # Derive deterministic fake values from hash nibbles so tests can assert.
    id_num = str(int(h[:7], 16))[:8].zfill(8)
    year = 2020 + (int(h[7], 16) % 6)
    month = 1 + (int(h[8], 16) % 12)
    day = 1 + (int(h[9], 16) % 28)
    fields = DeathCertificateFields(
        deceased_name=f"Stub Name {h[:4].upper()}",
        national_id=id_num,
        date_of_death=f"{year}-{month:02d}-{day:02d}",
        place_of_death=f"County {int(h[10], 16) % 47 + 1:03d}",
        cert_serial=f"CERT-{h[11:15].upper()}",
    )
    return fields, 0.0  # stub always reports 0.0 confidence


def _extract_id_stub(image_bytes: bytes) -> tuple[IdCorrectionFields, float]:
    h = _stub_hash(image_bytes)
    id_num = str(int(h[:7], 16))[:8].zfill(8)
    year = 1960 + (int(h[7], 16) % 50)
    month = 1 + (int(h[8], 16) % 12)
    day = 1 + (int(h[9], 16) % 28)
    fields = IdCorrectionFields(
        full_name=f"Stub Name {h[:4].upper()}",
        national_id=id_num,
        date_of_birth=f"{year}-{month:02d}-{day:02d}",
        gender="M" if int(h[10], 16) % 2 == 0 else "F",
        county=f"County {int(h[11], 16) % 47 + 1:03d}",
    )
    return fields, 0.0


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------


def extract(image_bytes: bytes, modality: OcrModality) -> OcrResponse:
    """Extract structured fields from a certificate image.

    Selects TesseractAdapter when tesseract-ocr is available; otherwise falls
    back to StubAdapter with a WARN log.
    """
    if len(image_bytes) == 0:
        raise ValueError("image_bytes must not be empty")

    try:
        image = Image.open(__import__("io").BytesIO(image_bytes))
    except Exception as exc:
        raise ValueError(f"invalid image: {exc}") from exc

    warnings: list[str] = []

    if TESSERACT_OK:
        if modality == OcrModality.death_certificate:
            fields, confidence = _extract_death_tesseract(image)
        else:
            fields, confidence = _extract_id_correction_tesseract(image)
        adapter = "tesseract"
    else:
        warnings.append(
            "tesseract unavailable; stub adapter used - results are deterministic placeholders"
        )
        if modality == OcrModality.death_certificate:
            fields, confidence = _extract_death_stub(image_bytes)
        else:
            fields, confidence = _extract_id_stub(image_bytes)
        adapter = "stub"

    return OcrResponse(
        modality=modality,
        confidence=confidence,
        extracted=fields,
        adapter=adapter,
        warnings=warnings,
    )
