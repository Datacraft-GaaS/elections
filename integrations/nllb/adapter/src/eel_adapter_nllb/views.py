"""Request/response models for the nllb adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	inputText: str = Field(..., description="inputText (PII B (Internal))")
	translatedText: str = Field(..., description="translatedText (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	inputText: str | None = None
	translatedText: str | None = None
