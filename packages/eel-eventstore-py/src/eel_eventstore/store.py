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

"""EventStore: append-only event log with optimistic concurrency and snapshots."""

from __future__ import annotations

import json
import logging
from collections.abc import Callable
from typing import Any

from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession

from eel_eventstore.models import AggregateSnapshot, StoredEvent
from eel_eventstore.protocols import Aggregate

logger = logging.getLogger(__name__)

EventStoreSchema = """
CREATE TABLE IF NOT EXISTS es_events (
	stream_id   TEXT        NOT NULL,
	seq         BIGINT      NOT NULL,
	hlc         TEXT        NOT NULL,
	event_type  TEXT        NOT NULL,
	payload     JSONB       NOT NULL,
	prev_hash   TEXT,
	created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
	PRIMARY KEY (stream_id, seq)
);

CREATE TABLE IF NOT EXISTS es_snapshots (
	stream_id   TEXT        NOT NULL PRIMARY KEY,
	seq         BIGINT      NOT NULL,
	hlc         TEXT        NOT NULL,
	state       JSONB       NOT NULL,
	created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
"""


class OptimisticConcurrencyError(Exception):
    """Raised when the expected stream version does not match the actual version."""

    def __init__(self, stream_id: str, expected: int, actual: int) -> None:
        self.stream_id = stream_id
        self.expected = expected
        self.actual = actual
        super().__init__(
            f"Stream {stream_id!r}: expected version {expected}, got {actual}."
        )


class EventStore:
    """Append-only event store backed by a SQL database via SQLAlchemy async sessions."""

    def __init__(self, session_factory: Callable[[], AsyncSession]) -> None:
        self._session_factory = session_factory

    async def append_events(
        self,
        stream_id: str,
        expected_version: int,
        events: list[StoredEvent],
    ) -> None:
        """Append events to a stream with optimistic concurrency control.

        Args:
                stream_id: Target stream identifier.
                expected_version: The max(seq) the caller expects currently in the stream.
                        Use 0 if the stream should not exist yet.
                events: Ordered list of StoredEvent to append.

        Raises:
                OptimisticConcurrencyError: Current max(seq) differs from expected_version.
        """
        assert stream_id, "stream_id must not be empty"
        assert events, "events list must not be empty"

        async with self._session_factory() as session:
            result = await session.execute(
                text(
                    "SELECT COALESCE(MAX(seq), 0) FROM es_events WHERE stream_id = :sid"
                ),
                {"sid": stream_id},
            )
            actual_version: int = result.scalar() or 0

            if actual_version != expected_version:
                raise OptimisticConcurrencyError(
                    stream_id, expected_version, actual_version
                )

            for event in events:
                await session.execute(
                    text(
                        "INSERT INTO es_events (stream_id, seq, hlc, event_type, payload, prev_hash)"
                        " VALUES (:stream_id, :seq, :hlc, :event_type, :payload::jsonb, :prev_hash)"
                    ),
                    {
                        "stream_id": event.stream_id,
                        "seq": event.seq,
                        "hlc": event.hlc,
                        "event_type": event.event_type,
                        "payload": json.dumps(event.payload),
                        "prev_hash": event.prev_hash,
                    },
                )
            await session.commit()

    async def load_events(
        self,
        stream_id: str,
        after_seq: int = 0,
    ) -> list[StoredEvent]:
        """Load events for a stream after a given sequence number (exclusive)."""
        async with self._session_factory() as session:
            result = await session.execute(
                text(
                    "SELECT stream_id, seq, hlc, event_type, payload, prev_hash"
                    " FROM es_events"
                    " WHERE stream_id = :sid AND seq > :after_seq"
                    " ORDER BY seq ASC"
                ),
                {"sid": stream_id, "after_seq": after_seq},
            )
            rows = result.fetchall()

        return [
            StoredEvent(
                stream_id=row.stream_id,
                seq=row.seq,
                hlc=row.hlc,
                event_type=row.event_type,
                payload=row.payload
                if isinstance(row.payload, dict)
                else json.loads(row.payload),
                prev_hash=row.prev_hash,
            )
            for row in rows
        ]

    async def save_snapshot(self, snapshot: AggregateSnapshot) -> None:
        """Upsert a snapshot for a stream."""
        async with self._session_factory() as session:
            await session.execute(
                text(
                    "INSERT INTO es_snapshots (stream_id, seq, hlc, state)"
                    " VALUES (:stream_id, :seq, :hlc, :state::jsonb)"
                    " ON CONFLICT (stream_id) DO UPDATE"
                    "   SET seq = EXCLUDED.seq, hlc = EXCLUDED.hlc, state = EXCLUDED.state,"
                    "       created_at = now()"
                ),
                {
                    "stream_id": snapshot.stream_id,
                    "seq": snapshot.seq,
                    "hlc": snapshot.hlc,
                    "state": json.dumps(snapshot.state),
                },
            )
            await session.commit()

    async def load_snapshot(self, stream_id: str) -> AggregateSnapshot | None:
        """Load the latest snapshot for a stream, or None if none exists."""
        async with self._session_factory() as session:
            result = await session.execute(
                text(
                    "SELECT stream_id, seq, hlc, state"
                    " FROM es_snapshots WHERE stream_id = :sid"
                ),
                {"sid": stream_id},
            )
            row = result.fetchone()

        if row is None:
            return None

        return AggregateSnapshot(
            stream_id=row.stream_id,
            seq=row.seq,
            hlc=row.hlc,
            state=row.state if isinstance(row.state, dict) else json.loads(row.state),
        )

    async def replay(self, aggregate: Aggregate, stream_id: str) -> None:
        """Replay events onto an aggregate, using snapshots when available.

        Loads the latest snapshot (if any), then applies all events after the snapshot seq.
        """
        snapshot = await self.load_snapshot(stream_id)
        after_seq = 0

        if snapshot is not None:
            # Restore aggregate state from snapshot via duck-typed attribute assignment.
            # Concrete Aggregate implementations must support this.
            aggregate.__dict__.update({"seq": snapshot.seq})
            if hasattr(aggregate, "restore_snapshot"):
                aggregate.restore_snapshot(snapshot)  # type: ignore[union-attr]
            after_seq = snapshot.seq

        events = await self.load_events(stream_id, after_seq=after_seq)
        for event in events:
            aggregate.apply(event)
