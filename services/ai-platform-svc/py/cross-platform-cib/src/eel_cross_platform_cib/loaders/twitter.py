"""Twitter / X data loader.

Data-sharing prerequisites:
  - Requires a signed X Developer Agreement (Academic Research or Pro track).
  - Only public tweets may be ingested; DMs are never ingested.
  - Author IDs are pseudonymised (SHA-256 + daily salt) before storage.
  - Data retention: 30 days post-election.
"""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from ..models import EngagementStats, Post, make_content_hash


class TwitterLoader:
    """Loader for X (Twitter) v2 API tweet objects."""

    platform = "twitter"

    def load(self, raw: dict[str, Any]) -> Post:
        """Normalise a Twitter v2 tweet object.

        Expected keys: id, text, author_id, created_at,
        public_metrics.{like_count, retweet_count, reply_count, impression_count},
        referenced_tweets (array with type=retweeted|quoted).
        """
        text = raw.get("text", "") or ""
        metrics = raw.get("public_metrics", {})

        # Detect shares (retweets / quotes).
        shared_from: str | None = None
        for ref in raw.get("referenced_tweets", []):
            if ref.get("type") in ("retweeted", "quoted"):
                shared_from = make_content_hash(ref.get("text", ref.get("id", "")))
                break

        ts_raw = raw.get("created_at", "")
        try:
            ts = datetime.fromisoformat(ts_raw.replace("Z", "+00:00"))
        except (ValueError, TypeError):
            ts = datetime.now(tz=timezone.utc)

        return Post(
            platform=self.platform,
            author_id=str(raw.get("author_id", "")),
            content_hash=make_content_hash(text),
            ts=ts,
            engagement_stats=EngagementStats(
                likes=metrics.get("like_count", 0),
                shares=metrics.get("retweet_count", 0),
                comments=metrics.get("reply_count", 0),
                views=metrics.get("impression_count", 0),
            ),
            shared_from=shared_from,
            post_id=str(raw.get("id", "")),
        )

    def load_batch(self, records: list[dict[str, Any]]) -> list[Post]:
        return [self.load(r) for r in records]
