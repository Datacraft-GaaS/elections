"""Pydantic v2 request/response models for the fp-dl-worker API."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class EmbedRequest(BaseModel):
    """POST /embed: raw image bytes (base64-encoded)."""

    model_config = ConfigDict(extra="forbid")

    image_b64: str = Field(
        ..., description="Base64-encoded 8-bit grayscale image (PNG or raw bytes)."
    )


class EmbedResponse(BaseModel):
    """POST /embed response: 384-dimensional float32 embedding."""

    model_config = ConfigDict(extra="forbid")

    embedding: list[float] = Field(
        ..., description="384-d L2-normalised embedding vector."
    )
    dim: int = Field(384, description="Embedding dimension.")


class MatchRequest(BaseModel):
    """POST /match: two images, returns cosine similarity."""

    model_config = ConfigDict(extra="forbid")

    probe_b64: str = Field(..., description="Base64-encoded probe image.")
    gallery_b64: str = Field(..., description="Base64-encoded gallery image.")


class MatchResponse(BaseModel):
    """POST /match response."""

    model_config = ConfigDict(extra="forbid")

    score: float = Field(
        ..., ge=0.0, le=1.0, description="Cosine similarity in [0, 1]."
    )
    probe_dim: int = Field(384)
    gallery_dim: int = Field(384)


class HealthResponse(BaseModel):
    """GET /health response."""

    model_config = ConfigDict(extra="forbid")

    status: str
    model_loaded: bool
    model_path: str | None = None
