"""Request/response models for the printer-mes adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	printOrderRef: str = Field(..., description="printOrderRef (PII B (Internal))")
	ballotQty: str = Field(..., description="ballotQty (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	printOrderRef: str | None = None
	ballotQty: str | None = None
