"""Pydantic models for the eel event store."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict


class StoredEvent(BaseModel):
    model_config = ConfigDict(extra="forbid", populate_by_name=True)

    stream_id: str
    seq: int
    hlc: str
    event_type: str
    payload: dict[str, Any]
    prev_hash: str | None = None


class AggregateSnapshot(BaseModel):
    model_config = ConfigDict(extra="forbid", populate_by_name=True)

    stream_id: str
    seq: int
    hlc: str
    state: dict[str, Any]
