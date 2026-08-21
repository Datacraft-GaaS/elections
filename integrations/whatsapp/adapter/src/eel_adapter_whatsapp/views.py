"""Request/response models for the whatsapp adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	to: str = Field(..., description="to (PII A (Restricted))")
	templateParams: str = Field(..., description="templateParams (PII B (Internal))")
	body: str = Field(..., description="body (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	to: str | None = None
	templateParams: str | None = None
	body: str | None = None
