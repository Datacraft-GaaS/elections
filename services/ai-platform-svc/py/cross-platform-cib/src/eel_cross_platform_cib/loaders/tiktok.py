"""TikTok data loader.

Data-sharing prerequisites:
  - Requires a signed TikTok Research API agreement (DSSC / academic track).
  - Only public videos and their metadata are ingested; private content is never accessed.
  - Author IDs are pseudonymised before storage.
  - Data retention: 30 days post-election.
"""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from ..models import EngagementStats, Post, make_content_hash


class TikTokLoader:
    """Loader for TikTok Research API video objects."""

    platform = "tiktok"

    def load(self, raw: dict[str, Any]) -> Post:
        """Normalise a TikTok Research API video record.

        Expected keys: id, author_id, video_description, create_time,
        statistics.{digg_count, share_count, comment_count, play_count},
        duet_from_id (if it's a duet/stitch).
        """
        text = raw.get("video_description", "") or ""
        stats = raw.get("statistics", {})

        shared_from: str | None = None
        if raw.get("duet_from_id"):
            shared_from = make_content_hash(str(raw["duet_from_id"]))

        ts_raw = raw.get("create_time")
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

        return Post(
            platform=self.platform,
            author_id=str(raw.get("author_id", "")),
            content_hash=make_content_hash(text),
            ts=ts,
            engagement_stats=EngagementStats(
                likes=stats.get("digg_count", 0),
                shares=stats.get("share_count", 0),
                comments=stats.get("comment_count", 0),
                views=stats.get("play_count", 0),
            ),
            shared_from=shared_from,
            post_id=str(raw.get("id", "")),
        )

    def load_batch(self, records: list[dict[str, Any]]) -> list[Post]:
        return [self.load(r) for r in records]
