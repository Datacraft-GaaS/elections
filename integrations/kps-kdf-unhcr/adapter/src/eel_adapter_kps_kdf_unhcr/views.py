"""Request/response models for the kps-kdf-unhcr adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	refugeeId: str = Field(..., description="refugeeId (PII A (Restricted))")
	campLocation: str = Field(..., description="campLocation (PII B (Internal))")
	nationalId: str = Field(..., description="nationalId (PII A (Restricted))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	refugeeId: str | None = None
	campLocation: str | None = None
	nationalId: str | None = None
