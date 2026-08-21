"""Request/response models for the ecourt adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	petitionerName: str = Field(..., description="petitionerName (PII A (Restricted))")
	caseNo: str = Field(..., description="caseNo (PII B (Internal))")
	documentContent: str = Field(..., description="documentContent (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	petitionerName: str | None = None
	caseNo: str | None = None
	documentContent: str | None = None
