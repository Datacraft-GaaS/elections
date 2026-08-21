"""FastAPI application for the EEL face ONNX worker sidecar.

Endpoints:
  POST /detect   - face detection
  POST /align    - face alignment to 112x112 canonical template
  POST /embed    - face embedding (512-d L2-normalised)
  POST /quality  - ISO 29794-5 quality assessment
  GET  /health   - liveness + model status
"""

from __future__ import annotations

import base64
import logging

from fastapi import FastAPI, HTTPException

from .inference import get_pool, stub_align, stub_detect, stub_embed, stub_quality
from .models import (
    AlignRequest,
    AlignResponse,
    DetectRequest,
    DetectResponse,
    DetectedFace,
    EmbedRequest,
    EmbedResponse,
    FiveLandmarks,
    HealthResponse,
    QualityRequest,
    QualityResponse,
)

_log = logging.getLogger(__name__)

app = FastAPI(
    title="EEL Face ONNX Worker",
    version="0.1.0",
    description="ONNX inference sidecar for the EEL ABIS face pipeline",
)


@app.get("/health", response_model=HealthResponse)
async def health() -> HealthResponse:
    pool = get_pool()
    return HealthResponse(status="ok", models_loaded=pool.models_loaded())


@app.post("/detect", response_model=DetectResponse)
async def detect(req: DetectRequest) -> DetectResponse:
    try:
        image_bytes = base64.b64decode(req.image_b64)
    except Exception as exc:
        raise HTTPException(status_code=422, detail=f"invalid base64: {exc}") from exc

    pool = get_pool()
    if pool.detector is not None:
        # TODO: run real ONNX inference.
        _log.info("EEL: ONNX detector available but real inference not yet wired")

    raw = stub_detect(image_bytes, req.width, req.height, req.conf_threshold)
    faces = [
        DetectedFace(
            bbox=f["bbox"],
            confidence=f["confidence"],
            landmarks=FiveLandmarks(**f["landmarks"]) if f.get("landmarks") else None,
        )
        for f in raw
    ]
    return DetectResponse(faces=faces, model_used="stub-hash")


@app.post("/align", response_model=AlignResponse)
async def align(req: AlignRequest) -> AlignResponse:
    try:
        image_bytes = base64.b64decode(req.image_b64)
    except Exception as exc:
        raise HTTPException(status_code=422, detail=f"invalid base64: {exc}") from exc

    aligned = stub_align(image_bytes, req.width, req.height)
    return AlignResponse(aligned_b64=base64.b64encode(aligned).decode())


@app.post("/embed", response_model=EmbedResponse)
async def embed(req: EmbedRequest) -> EmbedResponse:
    try:
        aligned_bytes = base64.b64decode(req.aligned_b64)
    except Exception as exc:
        raise HTTPException(status_code=422, detail=f"invalid base64: {exc}") from exc

    pool = get_pool()
    if pool.embedder is not None:
        _log.info("EEL: ONNX embedder available; real inference not yet wired")

    embedding = stub_embed(aligned_bytes)
    return EmbedResponse(
        embedding=embedding, model_used=f"stub-hash-{req.model_family}"
    )


@app.post("/quality", response_model=QualityResponse)
async def quality(req: QualityRequest) -> QualityResponse:
    try:
        image_bytes = base64.b64decode(req.image_b64)
    except Exception as exc:
        raise HTTPException(status_code=422, detail=f"invalid base64: {exc}") from exc

    scores = stub_quality(image_bytes, req.width, req.height)
    return QualityResponse(**scores)
