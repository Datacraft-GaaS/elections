from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict


class ModelLayer(str, Enum):
    KIT = "KIT"
    CENTRAL = "CENTRAL"
    CITIZEN = "CITIZEN"
    DISINFORMATION = "DISINFORMATION"
    INTERNAL = "INTERNAL"


class ModelStatus(str, Enum):
    REGISTERED = "REGISTERED"
    PENDING_IAB = "PENDING_IAB"
    IAB_NOTIFIED = "IAB_NOTIFIED"
    CCB_APPROVED = "CCB_APPROVED"
    ACTIVE = "ACTIVE"
    ROLLED_BACK = "ROLLED_BACK"


class ModelCard(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    id: str
    code: str
    name: str
    version: str
    layer: ModelLayer
    architecture: str
    training_data_description: str
    eval_accuracy: float
    eval_bias_flag: bool
    known_limitations: str
    human_in_loop: bool
    sovereign_hosting: bool
    model_card_url: str | None
    iab_notified_at: datetime | None
    ccb_approval_id: str | None
    deployed_at: datetime | None
    rollback_procedure: str
    is_active: bool
    status: ModelStatus
    registered_at: datetime


class ModelRegistrationRequest(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    code: str
    name: str
    version: str
    layer: ModelLayer
    architecture: str
    training_data_description: str
    eval_accuracy: float
    eval_bias_flag: bool
    known_limitations: str
    human_in_loop: bool
    sovereign_hosting: bool
    model_card_url: str | None = None
    iab_notified_at: datetime | None = None
    ccb_approval_id: str | None = None
    deployed_at: datetime | None = None
    rollback_procedure: str


class ModelActivateRequest(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    ccb_approval_id: str
    hsm_signature: str


class DecisionLogEntry(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    id: str
    model_id: str
    model_version: str
    interaction_type: str
    input_hash: str
    confidence_score: float
    model_output: dict[str, Any]
    human_decision: str | None
    human_actor_id: str | None
    human_decision_at: datetime | None
    decision_at: datetime


class DecisionLogRequest(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    model_id: str
    model_version: str
    interaction_type: str
    input_hash: str
    confidence_score: float
    model_output: dict[str, Any]
    human_decision: str | None = None
    human_actor_id: str | None = None
    human_decision_at: datetime | None = None


class PiiStripEvent(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    id: str
    model_id: str
    original_input_length: int
    stripped_input_hash: str
    pii_tokens_replaced: int
    stripped_at: datetime


class ModelCardPublication(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    id: str
    model_id: str
    version: str
    published_at: datetime
    card_hash: str
    languages: list[str]
