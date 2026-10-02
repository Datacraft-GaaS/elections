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

"""Facebook data loader.

Data-sharing prerequisites:
  - Requires a signed Facebook CrowdTangle / Content Library API agreement.
  - Data subject to Meta Research API Terms of Service.
  - All author identifiers must be pseudonymised before ingestion per GDPR Art. 25.
  - Retain for no longer than 30 days after election declaration (data minimisation).
"""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from ..models import EngagementStats, Post, make_content_hash


class FacebookLoader:
    """Loader for Facebook Content Library API records."""

    platform = "facebook"

    def load(self, raw: dict[str, Any]) -> Post:
        """Normalise a Facebook post record.

        Expected keys:
          id, from_id, message, created_time,
          shares.count, likes.summary.total_count, comments.summary.total_count
        """
        text = raw.get("message", "") or ""
        shared_from_id = (
            raw.get("shared_story", {}).get("id") if raw.get("shared_story") else None
        )
        shared_hash = (
            make_content_hash(raw["shared_story"]["message"])
            if shared_from_id and raw.get("shared_story", {}).get("message")
            else None
        )

        ts_raw = raw.get("created_time", "")
        try:
            ts = datetime.fromisoformat(ts_raw)
            if ts.tzinfo is None:
                ts = ts.replace(tzinfo=timezone.utc)
        except (ValueError, TypeError):
            ts = datetime.now(tz=timezone.utc)

        return Post(
            platform=self.platform,
            author_id=str(raw.get("from_id", "")),
            content_hash=make_content_hash(text),
            ts=ts,
            engagement_stats=EngagementStats(
                likes=raw.get("likes", {}).get("summary", {}).get("total_count", 0),
                shares=raw.get("shares", {}).get("count", 0),
                comments=raw.get("comments", {})
                .get("summary", {})
                .get("total_count", 0),
            ),
            shared_from=shared_hash,
            post_id=str(raw.get("id", "")),
        )

    def load_batch(self, records: list[dict[str, Any]]) -> list[Post]:
        return [self.load(r) for r in records]
