"""Legacy ABIS adapter models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Match1NRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    probe_template: str = Field(
        ..., description="Base64-encoded fingerprint template (PII Restricted)"
    )
    modality: str = Field("fingerprint", description="fingerprint | iris | face")
    threshold: float = Field(0.8, ge=0.0, le=1.0)
    request_id: str


class Match11Request(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    probe_template: str = Field(
        ..., description="Base64-encoded probe template (PII Restricted)"
    )
    reference_id: str = Field(
        ..., description="Registered voter ID for reference lookup"
    )
    modality: str = Field("fingerprint")
    request_id: str


class MatchResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str
    matched: bool
    score: float
    candidate_id: str | None = None
    error: str | None = None
