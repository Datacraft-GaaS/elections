"""Request/response models for the ppdt adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	partyName: str = Field(..., description="partyName (PII C (Public))")
	registrationNo: str = Field(..., description="registrationNo (PII B (Internal))")
	officerName: str = Field(..., description="officerName (PII A (Restricted))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	partyName: str | None = None
	registrationNo: str | None = None
	officerName: str | None = None
