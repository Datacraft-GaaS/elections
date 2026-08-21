"""Legacy Android shim models."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field


class LegacyPayload(BaseModel):
    """Documented legacy pre-CloudEvents kit payload format."""

    model_config = ConfigDict(extra="allow")

    kit_id: str = Field(..., description="Legacy kit device ID")
    op: str = Field(..., description="Operation code e.g. 'REG', 'AUTH', 'SYNC'")
    ts: str = Field(..., description="Timestamp ISO-8601")
    data: dict[str, Any] = Field(default_factory=dict)
    sig: str | None = Field(None, description="Legacy ECDSA signature hex")
    seq: int | None = None


class CloudEventEnvelope(BaseModel):
    """eel CloudEvents 1.0 envelope."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    specversion: str = "1.0"
    id: str
    source: str
    type: str
    time: str
    datacontenttype: str = "application/json"
    data: dict[str, Any]
    # eel platform extensions
    eel_shim_key_id: str | None = None
    eel_legacy_sig: str | None = None
    eel_shim_sig: str | None = None
