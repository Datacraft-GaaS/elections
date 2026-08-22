"""Telegram data loader.

Data-sharing prerequisites:
  - Requires access to Telegram's Bot API or TDLib via a registered research account.
  - Only public channels are monitored; private groups are never accessed.
  - Author (channel) IDs are pseudonymised before ingestion.
  - Data retention: 30 days post-election.
"""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from ..models import EngagementStats, Post, make_content_hash


class TelegramLoader:
    """Loader for Telegram public channel message objects (TDLib / Bot API)."""

    platform = "telegram"

    def load(self, raw: dict[str, Any]) -> Post:
        """Normalise a Telegram message record.

        Expected keys: id, chat_id, text, date (Unix timestamp),
        views, forwards, replies.count,
        forward_from_chat.id (if forwarded).
        """
        text = raw.get("text", "") or raw.get("caption", "") or ""
        ts_raw = raw.get("date")
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

        shared_from: str | None = None
        if fwd := raw.get("forward_from_chat"):
            shared_from = make_content_hash(str(fwd.get("id", "")) + text)

        return Post(
            platform=self.platform,
            author_id=str(raw.get("chat_id", "")),
            content_hash=make_content_hash(text),
            ts=ts,
            engagement_stats=EngagementStats(
                views=raw.get("views", 0),
                shares=raw.get("forwards", 0),
                comments=raw.get("replies", {}).get("count", 0)
                if raw.get("replies")
                else 0,
            ),
            shared_from=shared_from,
            post_id=str(raw.get("id", "")),
        )

    def load_batch(self, records: list[dict[str, Any]]) -> list[Post]:
        return [self.load(r) for r in records]
