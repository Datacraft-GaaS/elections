from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel, ConfigDict, Field


class HeadcountRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    station_code: str
    image_source: str
    captured_at: datetime


class HeadcountResult(BaseModel):
    model_config = ConfigDict(extra="forbid")
    result_id: str
    station_code: str
    headcount: int
    confidence: float = Field(ge=0.0, le=1.0)
    source: str
    processed_at: datetime
