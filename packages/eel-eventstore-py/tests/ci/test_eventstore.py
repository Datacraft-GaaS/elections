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

"""Tests for eel-eventstore-py."""

from __future__ import annotations

import asyncio
from contextlib import asynccontextmanager
from typing import Any
from unittest.mock import AsyncMock, MagicMock

import pytest

from eel_eventstore import (
    AggregateSnapshot,
    EventStore,
    EventStoreSchema,
    OptimisticConcurrencyError,
    StoredEvent,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_event(
    stream_id: str = "stream-1", seq: int = 1, event_type: str = "test.event"
) -> StoredEvent:
    return StoredEvent(
        stream_id=stream_id,
        seq=seq,
        hlc=f"1700000000000.{seq}.node1",
        event_type=event_type,
        payload={"seq": seq},
    )


class FakeRow:
    def __init__(self, **kwargs: Any) -> None:
        self.__dict__.update(kwargs)


def _make_store_with_scalar(scalar: int):
    """Return an EventStore whose session always reports scalar as max(seq)."""

    @asynccontextmanager
    async def factory():
        session = MagicMock()
        result = MagicMock()
        result.scalar.return_value = scalar
        session.execute = AsyncMock(return_value=result)
        session.commit = AsyncMock()
        yield session

    return EventStore(factory)


# ---------------------------------------------------------------------------
# OptimisticConcurrencyError
# ---------------------------------------------------------------------------


def test_optimistic_concurrency_error_attributes():
    err = OptimisticConcurrencyError("stream-x", expected=3, actual=5)
    assert err.stream_id == "stream-x"
    assert err.expected == 3
    assert err.actual == 5
    assert "stream-x" in str(err)


def test_append_events_raises_on_version_mismatch():
    """append_events must raise OptimisticConcurrencyError when version does not match."""
    store = _make_store_with_scalar(5)  # DB has version 5
    events = [_make_event(seq=6)]

    with pytest.raises(OptimisticConcurrencyError) as exc_info:
        asyncio.run(store.append_events("stream-1", expected_version=3, events=events))

    err = exc_info.value
    assert err.expected == 3
    assert err.actual == 5


def test_append_events_succeeds_on_matching_version():
    """append_events must not raise when version matches DB."""
    store = _make_store_with_scalar(2)  # DB has version 2
    events = [_make_event(seq=3)]
    asyncio.run(store.append_events("stream-1", expected_version=2, events=events))
    # No exception means success.


# ---------------------------------------------------------------------------
# replay
# ---------------------------------------------------------------------------


class CountingAggregate:
    """Simple aggregate that records applied events."""

    stream_id = "stream-1"
    seq = 0
    applied: list[StoredEvent]

    def __init__(self) -> None:
        self.applied = []

    def apply(self, event: StoredEvent) -> None:
        self.applied.append(event)
        self.seq = event.seq

    def snapshot(self) -> AggregateSnapshot:
        return AggregateSnapshot(
            stream_id=self.stream_id,
            seq=self.seq,
            hlc="h",
            state={"count": len(self.applied)},
        )


def test_replay_applies_events_in_order():
    """replay must apply events in ascending seq order."""
    event_rows = [
        FakeRow(
            stream_id="stream-1",
            seq=i,
            hlc=f"h.{i}",
            event_type="test.event",
            payload={"seq": i},
            prev_hash=None,
        )
        for i in range(1, 4)
    ]

    # Use a single shared session so both load_snapshot and load_events calls go to it.
    execute_results: list[MagicMock] = []

    # First call: load_snapshot -> fetchone returns None (no snapshot)
    snap_result = MagicMock()
    snap_result.fetchone.return_value = None
    execute_results.append(snap_result)

    # Second call: load_events -> fetchall returns rows
    events_result = MagicMock()
    events_result.fetchall.return_value = event_rows
    execute_results.append(events_result)

    session = MagicMock()
    idx = [0]

    async def execute_side(*args, **kwargs):
        r = execute_results[idx[0]]
        idx[0] += 1
        return r

    session.execute = AsyncMock(side_effect=execute_side)
    session.commit = AsyncMock()

    @asynccontextmanager
    async def factory():
        yield session

    store = EventStore(factory)
    agg = CountingAggregate()
    asyncio.run(store.replay(agg, "stream-1"))

    assert len(agg.applied) == 3
    seqs = [e.seq for e in agg.applied]
    assert seqs == sorted(seqs)


def test_replay_uses_snapshot_then_later_events():
    """replay must restore from snapshot and apply only events after snapshot seq."""
    snapshot_row = FakeRow(
        stream_id="stream-1",
        seq=5,
        hlc="h.5",
        state={"count": 5},
    )
    event_row = FakeRow(
        stream_id="stream-1",
        seq=6,
        hlc="h.6",
        event_type="test.event",
        payload={"seq": 6},
        prev_hash=None,
    )

    execute_results: list[MagicMock] = []

    snap_result = MagicMock()
    snap_result.fetchone.return_value = snapshot_row
    execute_results.append(snap_result)

    events_result = MagicMock()
    events_result.fetchall.return_value = [event_row]
    execute_results.append(events_result)

    session = MagicMock()
    idx = [0]

    async def execute_side(*args, **kwargs):
        r = execute_results[idx[0]]
        idx[0] += 1
        return r

    session.execute = AsyncMock(side_effect=execute_side)
    session.commit = AsyncMock()

    @asynccontextmanager
    async def factory():
        yield session

    store = EventStore(factory)
    agg = CountingAggregate()
    asyncio.run(store.replay(agg, "stream-1"))

    # Only the event after the snapshot should be applied.
    assert len(agg.applied) == 1
    assert agg.applied[0].seq == 6


# ---------------------------------------------------------------------------
# EventStoreSchema DDL
# ---------------------------------------------------------------------------


def test_eventstore_schema_contains_expected_tables():
    schema = EventStoreSchema.lower()
    assert "es_events" in schema
    assert "es_snapshots" in schema
    for col in ("stream_id", "seq", "hlc", "event_type", "payload", "prev_hash"):
        assert col in schema
