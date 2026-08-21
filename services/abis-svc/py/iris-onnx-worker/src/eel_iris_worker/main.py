import base64
import logging
from contextlib import asynccontextmanager
from typing import AsyncGenerator

import numpy as np
from fastapi import FastAPI, HTTPException

from . import inference
from .models import (
    Circle,
    EmbedResponse,
    EncodeResponse,
    HealthResponse,
    ImageRequest,
    LensResponse,
    QualityResponse,
    SegmentResponse,
)

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    logger.info("eel-iris-onnx-worker starting")
    inference.models_loaded()
    yield
    logger.info("eel-iris-onnx-worker stopped")


app = FastAPI(title="eel-iris-onnx-worker", lifespan=lifespan)


def _decode_image(req: ImageRequest) -> np.ndarray:
    """Decode base64 image to numpy array."""
    try:
        raw = base64.b64decode(req.image_b64)
    except Exception as exc:
        raise HTTPException(status_code=400, detail=f"Invalid base64: {exc}") from exc
    arr = np.frombuffer(raw, dtype=np.uint8).reshape(req.height, req.width)
    return arr


@app.post("/segment", response_model=SegmentResponse)
async def segment(req: ImageRequest) -> SegmentResponse:
    pixels = _decode_image(req)
    result = inference.hash_segment(pixels)
    mask_bytes = bytes([0xFF] * (req.width * req.height))
    mask_b64 = base64.b64encode(mask_bytes).decode()
    return SegmentResponse(
        pupil=Circle(cx=result["cx"], cy=result["cy"], radius=result["pupil_r"]),
        limbus=Circle(cx=result["cx"], cy=result["cy"], radius=result["limbus_r"]),
        iris_sclera_ratio=0.85,
        mask_b64=mask_b64,
    )


@app.post("/encode", response_model=EncodeResponse)
async def encode(req: ImageRequest) -> EncodeResponse:
    pixels = _decode_image(req)
    code_bytes, mask_bytes, quality = inference.hash_encode(pixels)
    return EncodeResponse(
        iris_code_b64=base64.b64encode(code_bytes).decode(),
        mask_b64=base64.b64encode(mask_bytes).decode(),
        quality=quality,
    )


@app.post("/quality", response_model=QualityResponse)
async def quality(req: ImageRequest) -> QualityResponse:
    pixels = _decode_image(req)
    q = inference.hash_quality(pixels)
    return QualityResponse(**q)


@app.post("/lens-detect", response_model=LensResponse)
async def lens_detect(req: ImageRequest) -> LensResponse:
    pixels = _decode_image(req)
    result = inference.hash_lens(pixels)
    return LensResponse(
        has_lens=bool(result["has_lens"]),
        kind=str(result["kind"]),
        confidence=float(result["confidence"]),
    )


@app.post("/embed", response_model=EmbedResponse)
async def embed(req: ImageRequest) -> EmbedResponse:
    pixels = _decode_image(req)
    vec = inference.hash_embed(pixels)
    return EmbedResponse(embedding=vec)


@app.get("/health", response_model=HealthResponse)
async def health() -> HealthResponse:
    return HealthResponse(status="ok", models_loaded=inference.models_loaded())
