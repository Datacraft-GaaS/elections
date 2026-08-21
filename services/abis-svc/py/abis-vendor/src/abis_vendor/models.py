from __future__ import annotations
from enum import StrEnum
from typing import Any
from pydantic import BaseModel, ConfigDict


class Modality(StrEnum):
    fingerprint = "fingerprint"
    face = "face"
    iris = "iris"


class FingerprintTemplate(BaseModel):
    model_config = ConfigDict(extra="forbid")
    modality: Modality = Modality.fingerprint
    data: str  # hex-encoded bytes
    quality: float
    vendor: str


class FaceTemplate(BaseModel):
    model_config = ConfigDict(extra="forbid")
    modality: Modality = Modality.face
    data: str  # hex-encoded bytes
    quality: float
    vendor: str
    embedding: list[float] | None = None


class IrisTemplate(BaseModel):
    model_config = ConfigDict(extra="forbid")
    modality: Modality = Modality.iris
    data: str  # hex-encoded bytes
    quality: float
    vendor: str


Template = FingerprintTemplate | FaceTemplate | IrisTemplate


class ExtractionRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    modality: Modality
    image_b64: str
    options: dict[str, Any] = {}


class ExtractionResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    template: FingerprintTemplate | FaceTemplate | IrisTemplate
    duration_ms: float


class MatchRequest11(BaseModel):
    model_config = ConfigDict(extra="forbid")
    probe: FingerprintTemplate | FaceTemplate | IrisTemplate
    gallery: FingerprintTemplate | FaceTemplate | IrisTemplate


class MatchRequestNx1(BaseModel):
    model_config = ConfigDict(extra="forbid")
    probe: FingerprintTemplate | FaceTemplate | IrisTemplate
    gallery_batch: list[FingerprintTemplate | FaceTemplate | IrisTemplate]


class Score(BaseModel):
    model_config = ConfigDict(extra="forbid")
    subject_id: str | None = None
    score: float


class MatchResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    scores: list[Score]
    duration_ms: float


class HealthStatus(BaseModel):
    model_config = ConfigDict(extra="forbid")
    ok: bool
    vendor: str
    mode: str


# Backward-compat models
class MatchOutcome(StrEnum):
    match = "match"
    no_match = "no_match"
    adjudicate = "adjudicate"
    insufficient_quality = "insufficient_quality"


class MatchRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    request_id: str
    probe_ref: str
    modality: str
    election_id: str


class MatchResult(BaseModel):
    model_config = ConfigDict(extra="forbid")
    request_id: str
    outcome: MatchOutcome
    confidence_score: float
    vendor_ref: str | None = None
    processed_at: str


class VendorConfig(BaseModel):
    model_config = ConfigDict(extra="forbid")
    adapter_url: str
    timeout_s: float = 30.0
    mock_mode: bool = True
