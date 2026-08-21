"""Pydantic v2 models for grievance-channels (WP-4.1)."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Literal

from pydantic import BaseModel, ConfigDict


class GrievanceCase(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    case_id: str
    channel: Literal["ussd", "sms", "whatsapp", "web", "kiosk", "api"]
    category: str
    severity: Literal["low", "medium", "high", "critical"]
    description_hash: str
    evidence_refs: list[str] = []
    status: Literal["open", "assigned", "escalated", "resolved", "closed"] = "open"
    assigned_to: str | None = None
    sla_deadline: datetime | None = None
    resolved_at: datetime | None = None
    created_at: datetime


class EvidencePackage(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    pkg_id: str
    case_id: str
    content_refs: list[str] = []
    chain_of_custody: dict[str, Any] = {}
    tlog_hash: str | None = None
    assembled_at: datetime


class WhistleblowerProfile(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    profile_id: str
    case_id: str
    pseudonym: str
    protection_level: Literal["standard", "enhanced", "high"]
    encrypted_contact: bytes | None = None
    created_at: datetime


class UssdSession(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    session_id: str
    phone_msisdn: str
    step: int = 0
    case_draft: dict[str, Any] = {}


class WhatsappWebhook(BaseModel):
    model_config = ConfigDict(extra="allow")

    object: str
    entry: list[dict[str, Any]] = []


class SlaPolicy(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    category: str
    severity: str
    response_hours: int
    resolution_hours: int


class ReferralPackage(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    case_id: str
    referral_type: Literal["dci", "odpp"]
    evidence_pkg_id: str
    submitted_at: datetime | None = None
