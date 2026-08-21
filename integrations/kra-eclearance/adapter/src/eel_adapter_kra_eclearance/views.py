"""Request/response models for the kra-eclearance adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	importerPin: str = Field(..., description="importerPin (PII A (Restricted))")
	entryRef: str = Field(..., description="entryRef (PII B (Internal))")
	dutyAmount: str = Field(..., description="dutyAmount (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	importerPin: str | None = None
	entryRef: str | None = None
	dutyAmount: str | None = None
