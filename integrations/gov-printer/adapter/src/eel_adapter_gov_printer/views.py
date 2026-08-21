"""Request/response models for the gov-printer adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Request(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	documentRef: str = Field(..., description="documentRef (PII B (Internal))")
	gazetteNoticeNo: str = Field(..., description="gazetteNoticeNo (PII B (Internal))")
	request_id: str = Field(..., description="UUIDv7 idempotency key")


class Response(BaseModel):
	model_config = ConfigDict(
		extra="forbid", validate_by_name=True, validate_by_alias=True
	)

	request_id: str
	ok: bool = True
	message: str = "ok"
	documentRef: str | None = None
	gazetteNoticeNo: str | None = None
