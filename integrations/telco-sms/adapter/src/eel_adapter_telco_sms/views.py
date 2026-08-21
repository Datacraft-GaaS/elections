"""SMS adapter models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class SendRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    to: list[str] = Field(..., description="MSISDN list e.g. +254...")
    message: str
    sender_id: str | None = None
    request_id: str = Field(..., description="UUIDv7 idempotency key")
    provider: str = Field(
        "at", description="'at' = Africa's Talking, 'smpp' = Safaricom SMPP"
    )


class SendResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str
    accepted: int
    rejected: int
    message_ids: list[str]


class ReceiveWebhook(BaseModel):
    model_config = ConfigDict(extra="allow")

    from_: str = Field(..., alias="from")
    to: str
    text: str
    date: str
    id: str
