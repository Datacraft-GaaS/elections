"""IVR Pydantic models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict


class AtVoicePayload(BaseModel):
    """Africa's Talking voice webhook payload (form-encoded)."""

    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    sessionId: str
    phoneNumber: str
    dtmfDigits: str | None = None
    isActive: str = "1"


class IvrSession(BaseModel):
    """In-memory IVR session state."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    session_id: str
    phone: str
    state: str = "main_menu"
    lang: str = "en"


class SsmlResponse(BaseModel):
    """SSML/text response to return to Africa's Talking."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    content: str
    is_terminal: bool = False
