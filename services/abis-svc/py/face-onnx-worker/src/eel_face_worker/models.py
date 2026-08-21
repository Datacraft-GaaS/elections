"""Pydantic v2 request/response models for the face ONNX worker API."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class DetectRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    # Base64-encoded RGB image bytes (row-major, 3 bytes/pixel).
    image_b64: str = Field(..., description="Base64-encoded RGB image")
    width: int = Field(..., gt=0)
    height: int = Field(..., gt=0)
    conf_threshold: float = Field(0.5, ge=0.0, le=1.0)
    iou_threshold: float = Field(0.4, ge=0.0, le=1.0)


class BoundingBox(BaseModel):
    model_config = ConfigDict(extra="forbid")

    x: float
    y: float
    width: float
    height: float


class FiveLandmarks(BaseModel):
    model_config = ConfigDict(extra="forbid")

    left_eye: tuple[float, float]
    right_eye: tuple[float, float]
    nose: tuple[float, float]
    left_mouth: tuple[float, float]
    right_mouth: tuple[float, float]


class DetectedFace(BaseModel):
    model_config = ConfigDict(extra="forbid")

    bbox: BoundingBox
    confidence: float
    landmarks: FiveLandmarks | None = None


class DetectResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    faces: list[DetectedFace]
    model_used: str


class AlignRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    image_b64: str
    width: int = Field(..., gt=0)
    height: int = Field(..., gt=0)
    landmarks: FiveLandmarks


class AlignResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    # Base64-encoded aligned 112x112 RGB image.
    aligned_b64: str
    width: int = 112
    height: int = 112


class EmbedRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    # Base64-encoded aligned 112x112 RGB image.
    aligned_b64: str
    model_family: str = Field("arcface", pattern="^(arcface|adaface|cosface)$")


class EmbedResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    embedding: list[float] = Field(..., description="L2-normalised 512-d embedding")
    model_used: str


class QualityRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    image_b64: str
    width: int = Field(..., gt=0)
    height: int = Field(..., gt=0)
    landmarks: FiveLandmarks | None = None


class QualityResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    sharpness: float
    resolution: float
    pose_yaw: float
    pose_pitch: float
    pose_roll: float
    illumination: float
    uniformity: float
    occlusion: float
    expression: float
    gaze: float
    overall: float


class HealthResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    status: str
    models_loaded: dict[str, bool]
