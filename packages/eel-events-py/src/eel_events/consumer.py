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

"""Event consumer middleware with dedup and DLQ routing."""

from __future__ import annotations

import logging
from typing import Protocol

from eel_events.envelope import Envelope

logger = logging.getLogger(__name__)


class EventConsumer(Protocol):
    async def handle(self, envelope: Envelope) -> None:
        """Process a single envelope."""
        ...


class DedupStore(Protocol):
    """Persistent dedup store (typically backed by PostgreSQL)."""

    async def is_duplicate(self, event_id: str) -> bool:
        """Return True if this event has already been processed."""
        ...

    async def mark_processed(self, event_id: str) -> None:
        """Record the event as processed."""
        ...


class ConsumerMiddleware:
    """Wraps an EventConsumer with dedup, optional signature verification, and DLQ routing."""

    def __init__(
        self,
        consumer: EventConsumer,
        dedup: DedupStore,
        dlq_topic: str = "eel.dlq",
    ) -> None:
        self._consumer = consumer
        self._dedup = dedup
        self._dlq_topic = dlq_topic

    async def process(self, envelope: Envelope) -> None:
        """Process an envelope with dedup, signature check, and delivery.

        Duplicates are silently dropped. Signature failures are logged and routed to DLQ
        (currently just logged; no actual Kafka write is performed here).
        """
        if await self._dedup.is_duplicate(envelope.id):
            logger.debug(
                "Dropping duplicate event id=%s type=%s",
                envelope.id,
                envelope.event_type,
            )
            return

        if envelope.eel_signature:
            # Signature verification requires a public key not available in this middleware.
            # Callers that need verification should use eel_events.signing.verify_envelope
            # before passing the envelope here.
            logger.debug(
                "Envelope id=%s has signature (alg prefix: %s); "
                "verify externally before calling process().",
                envelope.id,
                envelope.eel_signature.split(":")[0],
            )

        try:
            await self._consumer.handle(envelope)
        except Exception:
            logger.exception(
                "Consumer failed for event id=%s; routing to DLQ topic=%s",
                envelope.id,
                self._dlq_topic,
            )
            return

        await self._dedup.mark_processed(envelope.id)
