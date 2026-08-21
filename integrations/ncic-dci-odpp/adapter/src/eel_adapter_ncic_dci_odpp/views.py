"""Request/response models for the ncic-dci-odpp adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	nationalId: str = Field(..., description="nationalId (PII A (Restricted))")
	fullName: str = Field(..., description="fullName (PII A (Restricted))")
	criminalRecord: str = Field(..., description="criminalRecord (PII A (Restricted))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	nationalId: str | None = None
	fullName: str | None = None
	criminalRecord: str | None = None
