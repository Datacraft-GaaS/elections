"""Request/response models for the kra-itax adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	pin: str = Field(..., description="pin (PII A (Restricted))")
	taxStatus: str = Field(..., description="taxStatus (PII B (Internal))")
	fullName: str = Field(..., description="fullName (PII A (Restricted))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	pin: str | None = None
	taxStatus: str | None = None
	fullName: str | None = None
