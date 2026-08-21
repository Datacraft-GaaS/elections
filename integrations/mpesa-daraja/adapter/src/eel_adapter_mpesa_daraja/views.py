"""Request/response models for the mpesa-daraja adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	phoneNumber: str = Field(..., description="phoneNumber (PII A (Restricted))")
	amount: str = Field(..., description="amount (PII B (Internal))")
	accountReference: str = Field(..., description="accountReference (PII B (Internal))")
	transactionId: str = Field(..., description="transactionId (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	phoneNumber: str | None = None
	amount: str | None = None
	accountReference: str | None = None
	transactionId: str | None = None
