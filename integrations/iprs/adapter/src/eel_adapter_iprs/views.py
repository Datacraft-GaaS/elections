"""Request/response models for the IPRS adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class VerifyRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    national_id: str = Field(
        ..., description="14-digit national ID number (PII Restricted)"
    )
    full_name: str | None = Field(
        None, description="Full name for cross-check (PII Restricted)"
    )
    dob: str | None = Field(None, description="Date of birth ISO-8601 (PII Restricted)")
    request_id: str = Field(..., description="UUIDv7 idempotency key")


class VerifyResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str
    matched: bool
    duplicate: bool = False
    status_code: str
    message: str
