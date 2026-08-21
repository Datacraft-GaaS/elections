"""Request/response models for the ifmis adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	supplierId: str = Field(..., description="supplierId (PII B (Internal))")
	invoiceAmount: str = Field(..., description="invoiceAmount (PII B (Internal))")
	voteHead: str = Field(..., description="voteHead (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	supplierId: str | None = None
	invoiceAmount: str | None = None
	voteHead: str | None = None
