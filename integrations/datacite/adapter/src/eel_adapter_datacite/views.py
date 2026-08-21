"""Request/response models for the datacite adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	title: str = Field(..., description="title (PII C (Public))")
	doi: str = Field(..., description="doi (PII C (Public))")
	creators: str = Field(..., description="creators (PII C (Public))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	title: str | None = None
	doi: str | None = None
	creators: str | None = None
