from __future__ import annotations
from enum import StrEnum
from pydantic import BaseModel, ConfigDict


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
