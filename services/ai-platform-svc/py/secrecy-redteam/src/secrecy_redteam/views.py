from __future__ import annotations

from datetime import datetime
from typing import Literal

from pydantic import BaseModel, ConfigDict


class Engagement(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    engagement_id: str
    title: str
    scope: str
    status: Literal["planned", "active", "closed"]
    started_at: datetime
    closed_at: datetime | None = None


class Finding(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    finding_id: str
    engagement_id: str
    title: str
    severity: Literal["info", "low", "medium", "high", "critical"]
    description: str
    cve_ids: list[str] = []
    discovered_at: datetime


class Remediation(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    remediation_id: str
    finding_id: str
    status: Literal["open", "in_progress", "resolved", "accepted_risk"]
    owner: str
    due_date: datetime
    resolved_at: datetime | None = None


class IabAttestation(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    attestation_id: str
    engagement_id: str
    attested_by: str
    staging_clone_id: str
    attestation_text: str
    attested_at: datetime
