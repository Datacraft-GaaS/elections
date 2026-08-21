"""Fluent EventBuilder for constructing eel Envelopes."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

from uuid6 import uuid7

from eel_events.envelope import DataClass, Envelope
from eel_events.signing import SpiffeKeyProvider, sign_envelope


def _uuid7str() -> str:
    return str(uuid7())


class EventBuilder:
    """Fluent builder for eel Envelopes. All setters return self for chaining."""

    def __init__(self) -> None:
        self._kwargs: dict[str, Any] = {}

    def event_type(self, t: str) -> EventBuilder:
        self._kwargs["event_type"] = t
        return self

    def source(self, s: str) -> EventBuilder:
        self._kwargs["source"] = s
        return self

    def subject(self, s: str) -> EventBuilder:
        self._kwargs["subject"] = s
        return self

    def data(self, d: dict[str, Any]) -> EventBuilder:
        self._kwargs["data"] = d
        return self

    def stream_id(self, s: str) -> EventBuilder:
        self._kwargs["eel_stream_id"] = s
        return self

    def seq(self, n: int) -> EventBuilder:
        self._kwargs["eel_seq"] = n
        return self

    def hlc(self, h: str) -> EventBuilder:
        self._kwargs["eel_hlc"] = h
        return self

    def prev_hash(self, h: str) -> EventBuilder:
        self._kwargs["eel_prev_hash"] = h
        return self

    def correlation_id(self, c: str) -> EventBuilder:
        self._kwargs["eel_correlation_id"] = c
        return self

    def causation_id(self, c: str) -> EventBuilder:
        self._kwargs["eel_causation_id"] = c
        return self

    def election_id(self, e: str) -> EventBuilder:
        self._kwargs["eel_election_id"] = e
        return self

    def data_class(self, c: DataClass) -> EventBuilder:
        self._kwargs["eel_data_class"] = c
        return self

    def data_schema(self, s: str) -> EventBuilder:
        self._kwargs["dataschema"] = s
        return self

    def build_unsigned(self) -> Envelope:
        """Build an Envelope without a signature, generating id and time if not set."""
        kwargs = dict(self._kwargs)
        kwargs.setdefault("id", _uuid7str())
        kwargs.setdefault("time", datetime.now(UTC))
        return Envelope(**kwargs)

    async def build(self, key_provider: SpiffeKeyProvider) -> Envelope:
        """Build an Envelope signed with Ed25519 via key_provider.

        eel_producer_key_id is set before signing so it is covered by the signature.
        """
        private_key, key_id = await key_provider.get_signing_key()
        # Build with key_id included so it's part of the signed canonical bytes.
        kwargs = dict(self._kwargs)
        kwargs.setdefault("id", _uuid7str())
        kwargs.setdefault("time", datetime.now(UTC))
        kwargs["eel_producer_key_id"] = key_id
        envelope = Envelope(**kwargs)
        sig = sign_envelope(envelope, private_key)
        data = envelope.model_dump(by_alias=False)
        data["eel_signature"] = sig
        return Envelope(**data)
