"""Protocols for the eel event store."""

from __future__ import annotations

from typing import Protocol

from eel_eventstore.models import AggregateSnapshot, StoredEvent


class Aggregate(Protocol):
    stream_id: str
    seq: int

    def apply(self, event: StoredEvent) -> None:
        """Apply a stored event to advance aggregate state."""
        ...

    def snapshot(self) -> AggregateSnapshot:
        """Return a point-in-time snapshot of the aggregate state."""
        ...
