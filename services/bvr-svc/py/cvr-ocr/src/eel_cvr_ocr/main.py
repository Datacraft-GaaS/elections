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

"""FastAPI application entry point for eel-cvr-ocr (M35)."""

from __future__ import annotations

import logging
from contextlib import asynccontextmanager
from typing import AsyncIterator

from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.responses import JSONResponse

from eel_cvr_ocr.extract import TESSERACT_OK, extract
from eel_cvr_ocr.models import HealthResponse, OcrModality, OcrResponse

_LOG = logging.getLogger("eel_cvr_ocr.main")

VERSION = "0.1.0"


@asynccontextmanager
async def lifespan(_app: FastAPI) -> AsyncIterator[None]:
    _LOG.info("eel-cvr-ocr starting (tesseract_available=%s)", TESSERACT_OK)
    yield
    _LOG.info("eel-cvr-ocr shutting down")


app = FastAPI(
    title="eel-cvr-ocr",
    description="CVR field-officer OCR service for death certificate and ID-correction recognition (M35)",
    version=VERSION,
    lifespan=lifespan,
)


# ---------------------------------------------------------------------------
# Health
# ---------------------------------------------------------------------------


@app.get("/health", response_model=HealthResponse)
async def health() -> HealthResponse:
    return HealthResponse(
        status="ok",
        tesseract_available=TESSERACT_OK,
        version=VERSION,
    )


# ---------------------------------------------------------------------------
# OCR endpoints
# ---------------------------------------------------------------------------


async def _run_ocr(file: UploadFile, modality: OcrModality) -> OcrResponse:
    content = await file.read()
    if len(content) == 0:
        raise HTTPException(status_code=422, detail="uploaded file is empty")
    try:
        return extract(content, modality)
    except ValueError as exc:
        raise HTTPException(status_code=422, detail=str(exc)) from exc


@app.post("/ocr/death-certificate", response_model=OcrResponse)
async def ocr_death_certificate(
    file: UploadFile = File(..., description="JPEG/PNG scan of the death certificate"),
) -> OcrResponse:
    """Extract structured fields from a scanned death certificate.

    Returns: deceased_name, national_id, date_of_death, place_of_death, cert_serial.
    """
    return await _run_ocr(file, OcrModality.death_certificate)


@app.post("/ocr/id-correction", response_model=OcrResponse)
async def ocr_id_correction(
    file: UploadFile = File(..., description="JPEG/PNG scan of the ID correction form"),
) -> OcrResponse:
    """Extract structured fields from a scanned ID-correction form.

    Returns: full_name, national_id, date_of_birth, gender, county.
    """
    return await _run_ocr(file, OcrModality.id_correction)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main() -> None:
    import uvicorn

    uvicorn.run("eel_cvr_ocr.main:app", host="0.0.0.0", port=8091, reload=False)


if __name__ == "__main__":
    main()
