"""USSD adapter models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict


class UssdRequest(BaseModel):
    """Africa's Talking USSD callback payload."""

    model_config = ConfigDict(extra="allow")

    sessionId: str
    serviceCode: str
    phoneNumber: str
    text: str
    networkCode: str = ""


class UssdResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    response: str
    continues: bool = True
