"""Tests for eel-outbox-py."""

from __future__ import annotations

import asyncio
from unittest.mock import AsyncMock, MagicMock, call, patch

import pytest

from eel_events import Envelope
from eel_outbox import MAX_INLINE_BYTES, OutboxSchema, emit


def _make_envelope(**overrides) -> Envelope:
    defaults = dict(
        id="01900000-0000-7000-8000-000000000001",
        event_type="eel.test.created.v1",
        source="spiffe://example.org/svc/test",
        data={"key": "value"},
        eel_stream_id="01900000-0000-7000-8000-000000000002",
        eel_seq=0,
        eel_hlc="1700000000000.0.node1",
    )
    defaults.update(overrides)
    return Envelope(**defaults)


def _mock_session() -> MagicMock:
    session = MagicMock()
    session.execute = AsyncMock()
    return session


# ---------------------------------------------------------------------------
# emit basic
# ---------------------------------------------------------------------------


def test_emit_writes_to_session():
    """emit() must call session.execute with INSERT INTO outbox."""
    session = _mock_session()
    env = _make_envelope()

    loop = asyncio.get_event_loop()
    loop.run_until_complete(emit(session, env))

    assert session.execute.called
    call_args = session.execute.call_args
    sql_str = str(call_args[0][0])
    assert "outbox" in sql_str.lower()
    params = call_args[0][1]
    assert params["id"] == env.id
    assert params["event_type"] == env.event_type
    assert params["stream_id"] == env.eel_stream_id
    assert params["blob_ref"] is None
    assert params["payload"] is not None


# ---------------------------------------------------------------------------
# size guard
# ---------------------------------------------------------------------------


def test_emit_raises_when_large_payload_without_minio():
    """Payloads >1 MB must raise ValueError when no minio_client."""
    session = _mock_session()
    # Patch model_dump_json to return oversized bytes
    env = _make_envelope()
    big_json = "x" * (MAX_INLINE_BYTES + 1)

    with patch.object(Envelope, "model_dump_json", return_value=big_json):
        with pytest.raises(ValueError, match="minio_client"):
            asyncio.get_event_loop().run_until_complete(emit(session, env))


def test_emit_large_payload_uses_minio():
    """Payloads >1 MB are uploaded to object storage; payload column is None."""
    session = _mock_session()
    env = _make_envelope()
    big_json = "x" * (MAX_INLINE_BYTES + 1)

    minio_client = MagicMock()
    minio_client.put_object = AsyncMock()

    with patch.object(Envelope, "model_dump_json", return_value=big_json):
        asyncio.get_event_loop().run_until_complete(
            emit(session, env, minio_client=minio_client)
        )

    minio_client.put_object.assert_called_once()
    params = session.execute.call_args[0][1]
    assert params["payload"] is None
    assert params["blob_ref"] is not None
    assert params["blob_ref"].startswith("minio://")


# ---------------------------------------------------------------------------
# OutboxSchema DDL
# ---------------------------------------------------------------------------


def test_outbox_schema_contains_expected_columns():
    schema = OutboxSchema.lower()
    for col in (
        "id",
        "event_type",
        "stream_id",
        "payload",
        "blob_ref",
        "created_at",
        "published_at",
    ):
        assert col in schema, f"Column {col!r} not in OutboxSchema DDL"
