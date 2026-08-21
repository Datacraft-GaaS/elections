from __future__ import annotations

from datetime import datetime, timezone
from typing import Annotated

from pydantic import AfterValidator, BaseModel, ConfigDict


def _non_empty(v: str) -> str:
    assert v, "must not be empty"
    return v


NonEmpty = Annotated[str, AfterValidator(_non_empty)]


class BagInfo(BaseModel):
    """Key-value metadata stored in bag-info.txt."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    source_organization: str | None = None
    contact_name: str | None = None
    contact_email: str | None = None
    external_description: str | None = None
    bagging_date: str | None = None
    # Payload-Oxum: "{octetcount}.{streamcount}"
    payload_oxum: str | None = None
    # Arbitrary extra metadata fields passed by caller.
    extra: dict[str, str] = {}


class BagManifest(BaseModel):
    """Result returned by BagBuilder after creating a bag."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    bag_id: str
    bag_path: str
    bagit_version: str = "1.0"
    tag_file_character_encoding: str = "UTF-8"
    # Map from relative path (data/<filename>) to SHA-256 hex digest.
    manifest: dict[str, str]
    payload_oxum: str
    bagging_date: str
    created_at: datetime


class VerificationResult(BaseModel):
    """Result returned by BagVerifier.verify()."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    valid: bool
    bag_path: str
    errors: list[str] = []
    verified_at: datetime = None  # type: ignore[assignment]

    def model_post_init(self, __context: object) -> None:
        if self.verified_at is None:
            object.__setattr__(self, "verified_at", datetime.now(timezone.utc))
