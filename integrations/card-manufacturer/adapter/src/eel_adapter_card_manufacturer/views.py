"""Request/response models for the card-manufacturer adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	voterNo: str = Field(..., description="voterNo (PII A (Restricted))")
	fullName: str = Field(..., description="fullName (PII A (Restricted))")
	cardRef: str = Field(..., description="cardRef (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	voterNo: str | None = None
	fullName: str | None = None
	cardRef: str | None = None
