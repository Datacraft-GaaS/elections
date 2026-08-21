from pydantic import BaseModel, ConfigDict


class IcrField(BaseModel):
    model_config = ConfigDict(extra="forbid")
    field_name: str
    value: str
    confidence: float  # 0.0-1.0
    bbox: list[float]  # [x, y, w, h]


class IcrResult(BaseModel):
    model_config = ConfigDict(extra="forbid")
    form_id: str
    fields: list[IcrField]
    requires_review: bool  # True if any field confidence < 0.90
    model_version: str
