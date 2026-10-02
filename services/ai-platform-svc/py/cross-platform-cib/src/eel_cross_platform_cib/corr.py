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

"""Cross-platform correlator: identifies content mirrored across N platforms within T minutes.

A "mirror cluster" is a set of posts that share the same content_hash and appear on
at least two distinct platforms within a configurable time window T.
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass, field
from datetime import timedelta

from .models import Post


@dataclass
class MirrorCluster:
    """A group of posts with the same content mirrored across platforms."""

    content_hash: str
    posts: list[Post] = field(default_factory=list)

    @property
    def platforms(self) -> set[str]:
        return {p.platform for p in self.posts}

    @property
    def platform_count(self) -> int:
        return len(self.platforms)

    @property
    def is_cross_platform(self) -> bool:
        return self.platform_count >= 2

    @property
    def earliest_ts(self):  # noqa: ANN201
        return min(p.ts for p in self.posts) if self.posts else None

    @property
    def latest_ts(self):  # noqa: ANN201
        return max(p.ts for p in self.posts) if self.posts else None

    def window_minutes(self) -> float | None:
        if not self.posts or len(self.posts) < 2:
            return None
        earliest = self.earliest_ts
        latest = self.latest_ts
        assert earliest is not None and latest is not None
        delta = latest - earliest
        return delta.total_seconds() / 60.0


@dataclass
class CorrelatorResult:
    mirror_clusters: list[MirrorCluster] = field(default_factory=list)
    total_posts_analysed: int = 0

    @property
    def flagged_post_hashes(self) -> set[str]:
        """Content hashes of all posts in cross-platform mirror clusters."""
        return {c.content_hash for c in self.mirror_clusters if c.is_cross_platform}


class CrossPlatformCorrelator:
    """Correlates posts across platforms to identify CIB mirror clusters.

    A cluster is flagged if:
      - The same content_hash appears on >= min_platforms distinct platforms.
      - All appearances fall within a window_minutes time window.

    Args:
        window_minutes: Maximum time window in minutes for a mirror to count.
        min_platforms: Minimum number of distinct platforms for a cluster to be flagged.
    """

    def __init__(self, window_minutes: float = 60.0, min_platforms: int = 2) -> None:
        if window_minutes <= 0:
            raise ValueError("window_minutes must be positive")
        if min_platforms < 2:
            raise ValueError("min_platforms must be >= 2")
        self.window_minutes = window_minutes
        self.min_platforms = min_platforms

    def correlate(self, posts: list[Post]) -> CorrelatorResult:
        """Run correlation on a list of posts.

        Returns a CorrelatorResult with all detected mirror clusters.
        Only clusters that are cross-platform within the time window are included.
        """
        by_hash: dict[str, list[Post]] = defaultdict(list)
        for post in posts:
            by_hash[post.content_hash].append(post)

        clusters: list[MirrorCluster] = []
        for ch, group in by_hash.items():
            cluster = self._build_cluster(ch, group)
            if cluster is not None:
                clusters.append(cluster)

        return CorrelatorResult(
            mirror_clusters=clusters,
            total_posts_analysed=len(posts),
        )

    def _build_cluster(
        self, content_hash: str, posts: list[Post]
    ) -> MirrorCluster | None:
        """Build a MirrorCluster if the posts satisfy cross-platform + window criteria."""
        if len(posts) < 2:
            return None

        # Sort by timestamp to enable sliding-window check.
        sorted_posts = sorted(posts, key=lambda p: p.ts)
        window = timedelta(minutes=self.window_minutes)

        # Find the largest subset within the window that has >= min_platforms platforms.
        best: list[Post] = []
        for start_idx, start_post in enumerate(sorted_posts):
            in_window = [
                p for p in sorted_posts[start_idx:] if (p.ts - start_post.ts) <= window
            ]
            platforms = {p.platform for p in in_window}
            if len(platforms) >= self.min_platforms and len(in_window) > len(best):
                best = in_window

        if not best:
            return None

        platforms = {p.platform for p in best}
        if len(platforms) < self.min_platforms:
            return None

        return MirrorCluster(content_hash=content_hash, posts=best)
