"""eel-eventstore-py: Append-only event store with optimistic concurrency."""

from eel_eventstore.models import AggregateSnapshot, StoredEvent
from eel_eventstore.protocols import Aggregate
from eel_eventstore.store import (
    EventStore,
    EventStoreSchema,
    OptimisticConcurrencyError,
)

__all__ = [
    "StoredEvent",
    "AggregateSnapshot",
    "Aggregate",
    "EventStore",
    "OptimisticConcurrencyError",
    "EventStoreSchema",
]
