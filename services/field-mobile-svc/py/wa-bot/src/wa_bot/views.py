"""WhatsApp webhook Pydantic models."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict


class WaTextBody(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    body: str


class WaMessage(BaseModel):
    id: str
    from_: str
    to: str
    type: str
    text: WaTextBody | None = None

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, populate_by_name=True
    )


class WaChange(BaseModel):
    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    value: dict[str, Any]
    field: str


class WaEntry(BaseModel):
    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    id: str
    changes: list[WaChange]


class WhatsAppWebhookPayload(BaseModel):
    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    object: str
    entry: list[WaEntry]


class FlowResponse(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    text: str
    to: str
