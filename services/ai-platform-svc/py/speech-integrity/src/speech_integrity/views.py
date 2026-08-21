from __future__ import annotations

from datetime import datetime
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class ContentItem(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    item_id: str
    platform: str
    content_text: str
    author_id: str
    published_at: datetime
    media_urls: list[str] = Field(default_factory=list)


class ContentFlag(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    flag_id: str
    item_id: str
    flag_type: Literal["misinformation", "hate_speech", "incitement", "deepfake", "cib"]
    confidence: float = Field(ge=0.0, le=1.0)
    flagged_at: datetime
    analyst_decision: Literal["pending", "confirmed", "rejected"] = "pending"
    decision_at: datetime | None = None
    decision_by: str | None = None


class CibCluster(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    cluster_id: str
    account_ids: list[str]
    similarity_score: float = Field(ge=0.0, le=1.0)
    detected_at: datetime


class DeepfakeResult(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    result_id: str
    item_id: str
    is_deepfake: bool
    confidence: float = Field(ge=0.0, le=1.0)
    analyzed_at: datetime


class AnalystDecision(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    flag_id: str
    decision: Literal["confirmed", "rejected"]
    decision_by: str
    notes: str = ""
