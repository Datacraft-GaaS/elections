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

"""FastAPI service for deep-learning fingerprint embedding.

Endpoints:
  POST /embed   - accepts base64-encoded image, returns 384-d embedding.
  POST /match   - accepts two base64-encoded images, returns cosine similarity.
  GET  /health  - liveness/readiness probe.
"""

from __future__ import annotations

import base64
import logging
from contextlib import asynccontextmanager
from typing import AsyncGenerator

from fastapi import FastAPI, HTTPException, status

from .model import FingerprintEmbedder
from .models import (
    EmbedRequest,
    EmbedResponse,
    HealthResponse,
    MatchRequest,
    MatchResponse,
)

_log = logging.getLogger(__name__)

# Module-level embedder instance (initialised at startup).
_embedder: FingerprintEmbedder | None = None


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    global _embedder
    _embedder = FingerprintEmbedder()
    _log.info("FingerprintEmbedder ready (model_loaded=%s)", _embedder.model_loaded)
    yield
    _embedder = None


app = FastAPI(
    title="eel-fp-dl-worker",
    description="Deep-learning fingerprint embedding sidecar (DeepPrint-style ONNX).",
    version="0.1.0",
    lifespan=lifespan,
)


def _get_embedder() -> FingerprintEmbedder:
    if _embedder is None:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Embedder not initialised (startup incomplete).",
        )
    return _embedder


def _decode_image(b64: str) -> bytes:
    try:
        return base64.b64decode(b64)
    except Exception as exc:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_CONTENT,
            detail=f"Invalid base64 image data: {exc}",
        ) from exc


@app.post("/embed", response_model=EmbedResponse, summary="Embed a fingerprint image")
async def embed(req: EmbedRequest) -> EmbedResponse:
    """Decode a base64-encoded image and return a 384-d L2-normalised embedding."""
    embedder = _get_embedder()
    image_bytes = _decode_image(req.image_b64)
    emb = embedder.embed(image_bytes)
    return EmbedResponse(embedding=emb.tolist(), dim=len(emb))


@app.post(
    "/match", response_model=MatchResponse, summary="Match two fingerprint images"
)
async def match(req: MatchRequest) -> MatchResponse:
    """Embed both images and return cosine similarity in [0, 1]."""
    embedder = _get_embedder()
    probe_bytes = _decode_image(req.probe_b64)
    gallery_bytes = _decode_image(req.gallery_b64)
    e_probe = embedder.embed(probe_bytes)
    e_gallery = embedder.embed(gallery_bytes)
    score = embedder.cosine_similarity(e_probe, e_gallery)
    return MatchResponse(
        score=score, probe_dim=len(e_probe), gallery_dim=len(e_gallery)
    )


@app.get("/health", response_model=HealthResponse, summary="Health check")
async def health() -> HealthResponse:
    """Return service health and model status."""
    if _embedder is None:
        return HealthResponse(status="starting", model_loaded=False)
    return HealthResponse(
        status="ok",
        model_loaded=_embedder.model_loaded,
        model_path=_embedder.model_path_str,
    )
