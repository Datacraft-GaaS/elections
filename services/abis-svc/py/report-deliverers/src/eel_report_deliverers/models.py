"""Pydantic models for delivery requests and receipts."""

from __future__ import annotations

from datetime import datetime
from typing import Any

from pydantic import BaseModel, ConfigDict


class DeliveryRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    artifact_ref: str
    artifact_id: str
    subscription_id: str
    format: str
    sha256: str


class DeliveryReceipt(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    # Gov printer
    submission_id: str | None = None
    submitted_at: datetime | None = None
    # KBC SFTP
    remote_path: str | None = None
    sha256: str | None = None
    uploaded_at: datetime | None = None


class VerifyResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    submission_id: str
    status: str
    verified_at: datetime | None = None
    detail: dict[str, Any] | None = None
