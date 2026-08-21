"""Request/response models for the odpc-breach adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	dataSubjects: str = Field(..., description="dataSubjects (PII A (Restricted))")
	breachDescription: str = Field(..., description="breachDescription (PII B (Internal))")
	contactEmail: str = Field(..., description="contactEmail (PII A (Restricted))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	dataSubjects: str | None = None
	breachDescription: str | None = None
	contactEmail: str | None = None
