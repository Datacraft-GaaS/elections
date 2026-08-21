"""Request/response models for the nssf adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	nationalId: str = Field(..., description="nationalId (PII A (Restricted))")
	memberNo: str = Field(..., description="memberNo (PII A (Restricted))")
	contributionAmount: str = Field(..., description="contributionAmount (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	nationalId: str | None = None
	memberNo: str | None = None
	contributionAmount: str | None = None
