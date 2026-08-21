"""CloudEvents envelope model for the eel platform."""

from __future__ import annotations

from datetime import UTC, datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field

from eel_events.rfc8785 import canonicalize


class DataClass(str, Enum):
    A = "A"
    B = "B"
    C = "C"
    D = "D"
    E = "E"


class Envelope(BaseModel):
    model_config = ConfigDict(extra="forbid", populate_by_name=True)

    specversion: str = "1.0"
    id: str
    event_type: str = Field(alias="type")
    source: str
    subject: str | None = None
    time: datetime = Field(default_factory=lambda: datetime.now(UTC))
    datacontenttype: str = "application/json"
    dataschema: str | None = None
    data: dict[str, Any]

    eel_stream_id: str
    eel_seq: int
    eel_hlc: str
    eel_prev_hash: str | None = None
    eel_correlation_id: str | None = None
    eel_causation_id: str | None = None
    eel_producer_key_id: str | None = None
    eel_signature: str | None = None
    eel_election_id: str | None = None
    eel_data_class: DataClass = DataClass.B

    def canonical_bytes(self) -> bytes:
        """Serialize to canonical JSON bytes, excluding eel_signature.

        Uses RFC 8785 key sorting (unicode code point order).
        """
        d = self.model_dump(by_alias=True, mode="json")
        d.pop("eel_signature", None)
        return canonicalize(d)
