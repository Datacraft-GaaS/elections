"""USSD request/response models (WP-4.7, M13)."""

from __future__ import annotations

from enum import IntEnum

from pydantic import BaseModel, ConfigDict


class UssdRequest(BaseModel):
    """Africa's Talking USSD webhook payload (form-encoded fields)."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    sessionId: str
    phoneNumber: str
    networkCode: str
    serviceCode: str
    text: str = ""


class UssdResponse(BaseModel):
    """Menu response returned to the gateway."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    text: str
    is_final: bool = False


class MenuStep(IntEnum):
    MAIN = 1
    VERIFY_TOKEN_PART1 = 2
    VERIFY_TOKEN_PART2 = 3
    RESULTS = 4
    GRIEVANCE_TYPE = 5
    GRIEVANCE_CONFIRM = 6


# Maximum characters per USSD page (FR-M13-029)
USSD_MAX_CHARS = 182
