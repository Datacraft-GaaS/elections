# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Tests for eel-events-py."""

from __future__ import annotations

import asyncio
import json
from datetime import UTC, datetime
from unittest.mock import AsyncMock

import nacl.signing

from eel_events import (
    ConsumerMiddleware,
    DataClass,
    Envelope,
    EventBuilder,
    canonicalize,
)
from eel_events.signing import sign_envelope, verify_envelope

# Fixed timestamp so canonical bytes are stable across _make_envelope calls.
_FIXED_TIME = datetime(2024, 1, 1, 0, 0, 0, tzinfo=UTC)


def _make_envelope(**overrides) -> Envelope:
    defaults = dict(
        id="01900000-0000-7000-8000-000000000001",
        event_type="eel.test.created.v1",
        source="spiffe://example.org/svc/test",
        data={"key": "value"},
        eel_stream_id="01900000-0000-7000-8000-000000000002",
        eel_seq=0,
        eel_hlc="1700000000000.0.node1",
        time=_FIXED_TIME,
    )
    defaults.update(overrides)
    return Envelope(**defaults)


# ---------------------------------------------------------------------------
# EventBuilder
# ---------------------------------------------------------------------------


def test_build_unsigned_returns_valid_envelope():
    env = (
        EventBuilder()
        .event_type("eel.order.placed.v1")
        .source("spiffe://example.org/svc/orders")
        .data({"order_id": "abc"})
        .stream_id("01900000-0000-7000-8000-000000000099")
        .seq(0)
        .hlc("1700000000000.0.node1")
        .build_unsigned()
    )
    assert isinstance(env, Envelope)
    assert env.event_type == "eel.order.placed.v1"
    assert env.specversion == "1.0"
    assert env.datacontenttype == "application/json"
    assert env.eel_data_class == DataClass.B
    assert env.eel_signature is None
    assert env.id  # generated


def test_build_unsigned_with_data_class():
    env = (
        EventBuilder()
        .event_type("eel.x.v1")
        .source("spiffe://example.org/s")
        .data({})
        .stream_id("sid")
        .seq(1)
        .hlc("h")
        .data_class(DataClass.C)
        .build_unsigned()
    )
    assert env.eel_data_class == DataClass.C


# ---------------------------------------------------------------------------
# canonical_bytes
# ---------------------------------------------------------------------------


def test_canonical_bytes_excludes_signature():
    env = _make_envelope(eel_signature="ed25519:abc123")
    cb = env.canonical_bytes()
    parsed = json.loads(cb.decode())
    assert "eel_signature" not in parsed


def test_canonical_bytes_deterministic():
    env = _make_envelope()
    assert env.canonical_bytes() == env.canonical_bytes()


def test_canonical_bytes_contains_event_type_as_type():
    env = _make_envelope()
    cb = env.canonical_bytes()
    parsed = json.loads(cb.decode())
    # by_alias=True means "type" is the key, not "event_type"
    assert "type" in parsed
    assert "event_type" not in parsed


# ---------------------------------------------------------------------------
# sign / verify
# ---------------------------------------------------------------------------


def test_sign_verify_roundtrip():
    signing_key = nacl.signing.SigningKey.generate()
    private_seed = bytes(signing_key)
    public_key = bytes(signing_key.verify_key)

    env = _make_envelope()
    sig = sign_envelope(env, private_seed)
    assert sig.startswith("ed25519:")

    # Must use the exact same envelope fields for verification - add signature to same envelope.
    signed_env = _make_envelope(eel_signature=sig)
    assert verify_envelope(signed_env, public_key) is True


def test_verify_wrong_key_returns_false():
    signing_key = nacl.signing.SigningKey.generate()
    private_seed = bytes(signing_key)

    env = _make_envelope()
    sig = sign_envelope(env, private_seed)
    signed_env = _make_envelope(eel_signature=sig)

    wrong_key = bytes(nacl.signing.SigningKey.generate().verify_key)
    assert verify_envelope(signed_env, wrong_key) is False


def test_verify_no_signature_returns_false():
    env = _make_envelope()
    public_key = bytes(nacl.signing.SigningKey.generate().verify_key)
    assert verify_envelope(env, public_key) is False


# ---------------------------------------------------------------------------
# async build with key provider
# ---------------------------------------------------------------------------


def test_build_with_key_provider():
    signing_key = nacl.signing.SigningKey.generate()
    private_seed = bytes(signing_key)
    public_key = bytes(signing_key.verify_key)
    key_id = "key-001"

    class FakeKeyProvider:
        async def get_signing_key(self) -> tuple[bytes, str]:
            return private_seed, key_id

    loop = asyncio.new_event_loop()
    env = loop.run_until_complete(
        EventBuilder()
        .event_type("eel.x.v1")
        .source("spiffe://example.org/s")
        .data({})
        .stream_id("sid")
        .seq(0)
        .hlc("h")
        .build(FakeKeyProvider())
    )
    loop.close()
    assert env.eel_signature is not None
    assert env.eel_producer_key_id == key_id
    assert verify_envelope(env, public_key) is True


# ---------------------------------------------------------------------------
# ConsumerMiddleware dedup
# ---------------------------------------------------------------------------


def test_middleware_dedup_skips_duplicate():
    """Duplicate event must be silently dropped (consumer.handle not called)."""

    class AlwaysDup:
        async def is_duplicate(self, event_id: str) -> bool:
            return True

        async def mark_processed(self, event_id: str) -> None:
            return None

    consumer_handle = AsyncMock()

    class MockConsumer:
        async def handle(self, envelope: Envelope) -> None:
            await consumer_handle(envelope)

    mw = ConsumerMiddleware(MockConsumer(), AlwaysDup())
    asyncio.run(mw.process(_make_envelope()))
    consumer_handle.assert_not_called()


def test_middleware_marks_processed_after_handle():
    mark_processed = AsyncMock()

    class FakeDedup:
        async def is_duplicate(self, event_id: str) -> bool:
            return False

        async def mark_processed(self, event_id: str) -> None:
            await mark_processed(event_id)

    class NullConsumer:
        async def handle(self, envelope: Envelope) -> None:
            return None

    mw = ConsumerMiddleware(NullConsumer(), FakeDedup())
    env = _make_envelope()
    asyncio.run(mw.process(env))
    mark_processed.assert_called_once_with(env.id)


# ---------------------------------------------------------------------------
# Envelope JSON round-trip
# ---------------------------------------------------------------------------


def test_envelope_json_roundtrip():
    env = _make_envelope()
    serialized = env.model_dump_json(by_alias=True)
    restored = Envelope.model_validate_json(serialized)
    assert restored.id == env.id
    assert restored.event_type == env.event_type
    assert restored.eel_hlc == env.eel_hlc


def test_envelope_accepts_type_alias_as_key():
    """Envelope must accept 'type' as the key (wire format)."""
    data = {
        "id": "01900000-0000-7000-8000-000000000001",
        "type": "eel.test.v1",
        "source": "spiffe://example.org/s",
        "data": {},
        "eel_stream_id": "sid",
        "eel_seq": 0,
        "eel_hlc": "h",
    }
    env = Envelope.model_validate(data)
    assert env.event_type == "eel.test.v1"


# ---------------------------------------------------------------------------
# canonicalize standalone
# ---------------------------------------------------------------------------


def test_canonicalize_sorts_keys():
    result = json.loads(canonicalize({"b": 1, "a": 2}).decode())
    assert list(result.keys()) == ["a", "b"]
