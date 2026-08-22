"""WhatsApp Broadcast data loader.

Data-sharing prerequisites:
  - Requires a signed Meta WhatsApp Business API research agreement.
  - Only broadcast-channel public posts are ingested; personal chats are never accessed.
  - Author identifiers are hashed (SHA-256 + daily salt) before ingestion.
  - Content is treated as sensitive PII; retention is 30 days post-election.
  - Legal basis: legitimate interest in election integrity, documented in DPA register.
"""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from ..models import EngagementStats, Post, make_content_hash


class WhatsAppBroadcastLoader:
    """Loader for WhatsApp Broadcast channel records (public channels only)."""

    platform = "whatsapp_broadcast"

    def load(self, raw: dict[str, Any]) -> Post:
        """Normalise a WhatsApp broadcast post record.

        Expected keys: id, channel_id, text, timestamp,
        forward_count, reaction_count, view_count,
        forwarded_from_hash (if forwarded).
        """
        text = raw.get("text", "") or ""
        ts_raw = raw.get("timestamp")
        if isinstance(ts_raw, int):
            ts = datetime.fromtimestamp(ts_raw, tz=timezone.utc)
        elif isinstance(ts_raw, str):
            try:
                ts = datetime.fromisoformat(ts_raw)
                if ts.tzinfo is None:
                    ts = ts.replace(tzinfo=timezone.utc)
            except ValueError:
                ts = datetime.now(tz=timezone.utc)
        else:
            ts = datetime.now(tz=timezone.utc)

        shared_from: str | None = raw.get("forwarded_from_hash")

        return Post(
            platform=self.platform,
            author_id=str(raw.get("channel_id", "")),
            content_hash=make_content_hash(text),
            ts=ts,
            engagement_stats=EngagementStats(
                likes=raw.get("reaction_count", 0),
                shares=raw.get("forward_count", 0),
                views=raw.get("view_count", 0),
            ),
            shared_from=shared_from,
            post_id=str(raw.get("id", "")),
        )

    def load_batch(self, records: list[dict[str, Any]]) -> list[Post]:
        return [self.load(r) for r in records]
