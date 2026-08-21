from __future__ import annotations
import os
import time
from contextlib import asynccontextmanager
from typing import Any
from fastapi import FastAPI, HTTPException
from .models import (
    ExtractionRequest,
    ExtractionResponse,
    FaceTemplate,
    FingerprintTemplate,
    HealthStatus,
    IrisTemplate,
    MatchRequest11,
    MatchRequestNx1,
    MatchResponse,
    Modality,
    Score,
)
import structlog

_log = structlog.get_logger()

_adapter: Any = None


@asynccontextmanager
async def lifespan(app: FastAPI):  # type: ignore[type-arg]
    global _adapter
    mode = os.environ.get("EEL_VENDOR_MODE", "mock")
    if mode == "mock":
        from .adapters.mock import MockAdapter

        _adapter = MockAdapter()
        _log.info("service.adapter_loaded", mode="mock")
    elif mode == "onnx":
        model_path = os.environ.get("EEL_ONNX_MODEL_PATH", "")
        if not model_path:
            raise RuntimeError("EEL_ONNX_MODEL_PATH must be set for onnx mode")
        from .adapters.onnx_face import OnnxFaceAdapter

        _adapter = OnnxFaceAdapter(model_path)
        _log.info("service.adapter_loaded", mode="onnx", model_path=model_path)
    elif mode == "py":
        from .adapters.py_fingerprint import PyFingerprintAdapter

        _adapter = PyFingerprintAdapter()
        _log.info("service.adapter_loaded", mode="py")
    else:
        raise RuntimeError(f"Unknown EEL_VENDOR_MODE: {mode!r}")
    yield
    _adapter = None


app = FastAPI(title="eel-abis-vendor", lifespan=lifespan)


@app.post("/extract/{modality}", response_model=ExtractionResponse)
async def extract(modality: str, body: ExtractionRequest) -> ExtractionResponse:
    if body.modality.value != modality:
        raise HTTPException(
            status_code=422,
            detail=f"Body modality {body.modality!r} does not match route /{modality}",
        )
    t0 = time.monotonic()
    if modality == Modality.fingerprint:
        template: FingerprintTemplate | FaceTemplate | IrisTemplate = (
            _adapter.extract_fingerprint(body.image_b64)
        )
    elif modality == Modality.face:
        template = _adapter.extract_face(body.image_b64)
    elif modality == Modality.iris:
        template = _adapter.extract_iris(body.image_b64)
    else:
        raise HTTPException(status_code=422, detail=f"Unknown modality: {modality!r}")
    duration_ms = (time.monotonic() - t0) * 1000.0
    return ExtractionResponse(template=template, duration_ms=duration_ms)


@app.post("/match/1x1", response_model=MatchResponse)
async def match_1x1(body: MatchRequest11) -> MatchResponse:
    t0 = time.monotonic()
    score: Score = _adapter.match_1x1(body.probe, body.gallery)
    duration_ms = (time.monotonic() - t0) * 1000.0
    return MatchResponse(scores=[score], duration_ms=duration_ms)


@app.post("/match/1xN", response_model=MatchResponse)
async def match_1xn(body: MatchRequestNx1) -> MatchResponse:
    t0 = time.monotonic()
    scores: list[Score] = _adapter.match_1xN(body.probe, body.gallery_batch)
    duration_ms = (time.monotonic() - t0) * 1000.0
    return MatchResponse(scores=scores, duration_ms=duration_ms)


@app.get("/health", response_model=HealthStatus)
async def health() -> HealthStatus:
    return _adapter.health()
