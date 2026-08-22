"""Pydantic v2 models for eel-rfid-reader-driver."""

from __future__ import annotations

from enum import Enum
from typing import Annotated

from pydantic import AfterValidator, BaseModel, ConfigDict, Field


def _non_empty(v: str) -> str:
    assert v.strip(), "must not be blank"
    return v


NonBlank = Annotated[str, AfterValidator(_non_empty)]


class TagState(str, Enum):
    """EPC Gen2v2 tag operational states as seen by the reader driver."""

    ready = "ready"
    challenged = "challenged"
    verified = "verified"
    tampered = "tampered"
    unknown = "unknown"


class ReaderRegistration(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    reader_id: NonBlank = Field(..., description="Reader serial / unique ID")
    svid: NonBlank = Field(
        ..., description="SPIFFE SVID (spiffe://eel.local/reader/<serial>)"
    )
    location: NonBlank = Field(..., description="Physical location label")
    reader_type: NonBlank = Field(
        ..., description="impinj | zebra | cipherlab | synthetic"
    )
    firmware_version: str = Field(
        default="", description="Reader firmware version string"
    )


class ReadRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    tag_epc: NonBlank = Field(..., description="EPC (hex string) of the scanned tag")
    rssi_dbm: float | None = Field(default=None, description="RSSI in dBm")
    read_ts: str | None = Field(default=None, description="ISO-8601 timestamp of read")
    tamper_indicator: bool = Field(
        default=False, description="Vendor tamper-indicator bit"
    )


class BatchReadRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    reads: list[ReadRequest] = Field(..., min_length=1, max_length=1000)


class ChallengeRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    tag_epc: NonBlank = Field(..., description="EPC of the tag to challenge")


class ReadResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str = Field(..., description="UUIDv7 idempotency key")
    tag_epc: str
    accepted: bool = True
    message: str = "ok"
    tag_state: TagState = TagState.unknown


class VerifyResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    tag_epc: str
    challenge_valid: bool
    tamper_detected: bool
    message: str = "ok"


class ChallengeResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    tag_epc: str
    nonce_hex: str = Field(..., description="16-byte nonce as hex")
    challenge_hex: str = Field(..., description="HMAC-SHA256(key, nonce||epc) as hex")


class ReaderStatus(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    reader_id: str
    healthy: bool
    last_read_ts: str | None = None
    tag_count: int = 0
    adapter_type: str = "unknown"
