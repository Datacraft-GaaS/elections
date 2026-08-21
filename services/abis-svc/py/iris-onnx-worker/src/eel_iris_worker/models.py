from pydantic import BaseModel, ConfigDict


class ImageRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    image_b64: str  # base64-encoded PNG/JPEG bytes
    width: int
    height: int


class Circle(BaseModel):
    model_config = ConfigDict(extra="forbid")
    cx: float
    cy: float
    radius: float


class SegmentResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    pupil: Circle
    limbus: Circle
    iris_sclera_ratio: float
    mask_b64: str  # base64-encoded binary mask (width*height bytes)


class EncodeResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    iris_code_b64: str  # base64-encoded 256-byte IrisCode
    mask_b64: str  # base64-encoded 256-byte mask
    quality: float


class QualityResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    sharpness: float
    contrast: float
    dilation: float
    gaze_angle_deg: float
    occlusion_fraction: float
    overall: float


class LensResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    has_lens: bool
    kind: str  # "none" | "transparent" | "coloured" | "patterned"
    confidence: float


class EmbedResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    embedding: list[float]  # 256 floats, L2-normalised


class HealthResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    status: str
    models_loaded: dict[str, bool]
