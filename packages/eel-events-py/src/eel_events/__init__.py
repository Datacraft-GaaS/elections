"""eel-events-py: CloudEvents envelope, signing, and consumer middleware."""

from eel_events.builder import EventBuilder
from eel_events.consumer import ConsumerMiddleware, DedupStore, EventConsumer
from eel_events.envelope import DataClass, Envelope
from eel_events.karapace import KarapaceClient
from eel_events.rfc8785 import canonicalize
from eel_events.signing import SpiffeKeyProvider

__all__ = [
    "Envelope",
    "DataClass",
    "EventBuilder",
    "SpiffeKeyProvider",
    "KarapaceClient",
    "EventConsumer",
    "DedupStore",
    "ConsumerMiddleware",
    "canonicalize",
]
