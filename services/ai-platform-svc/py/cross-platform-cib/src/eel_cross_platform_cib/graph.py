"""Heterogeneous social graph builder for CIB detection.

Builds a graph with three node types:
  - Post: one node per unique (platform, content_hash) pair.
  - Author: one node per (platform, author_id) pair.
  - Platform: one node per platform name.

Edge types:
  - AUTHORED: Author -> Post
  - ON_PLATFORM: Post -> Platform
  - SHARED_CONTENT: Post -> Post (probe shared_from target)
  - CROSS_PLATFORM_MIRROR: Post -> Post (same content_hash, different platform)
  - ENGAGEMENT: Post -> Post (co-engagement signal, placeholder for future)
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass, field
from typing import Any

import networkx as nx

from .models import Post


@dataclass
class GraphStats:
    post_nodes: int = 0
    author_nodes: int = 0
    platform_nodes: int = 0
    authored_edges: int = 0
    on_platform_edges: int = 0
    shared_content_edges: int = 0
    cross_platform_mirror_edges: int = 0


def _post_node_id(post: Post) -> str:
    return f"post:{post.platform}:{post.content_hash}"


def _author_node_id(post: Post) -> str:
    return f"author:{post.platform}:{post.author_id}"


def _platform_node_id(platform: str) -> str:
    return f"platform:{platform}"


def build_graph(posts: list[Post]) -> tuple[nx.MultiDiGraph, GraphStats]:
    """Build the heterogeneous social graph from a list of unified posts.

    Returns a directed multigraph and stats about the constructed graph.
    The graph is deterministic for a given input list (node/edge ordering).
    """
    G: nx.MultiDiGraph = nx.MultiDiGraph()
    stats = GraphStats()

    # Index posts by content_hash for mirror detection.
    content_to_posts: dict[str, list[Post]] = defaultdict(list)
    for post in posts:
        content_to_posts[post.content_hash].append(post)

    # 1. Add nodes.
    for post in posts:
        pid = _post_node_id(post)
        if not G.has_node(pid):
            G.add_node(
                pid,
                node_type="post",
                platform=post.platform,
                content_hash=post.content_hash,
                ts=post.ts.isoformat(),
                author_id=post.author_id,
            )
            stats.post_nodes += 1

        aid = _author_node_id(post)
        if not G.has_node(aid):
            G.add_node(
                aid,
                node_type="author",
                platform=post.platform,
                author_id=post.author_id,
            )
            stats.author_nodes += 1

        plid = _platform_node_id(post.platform)
        if not G.has_node(plid):
            G.add_node(plid, node_type="platform", platform=post.platform)
            stats.platform_nodes += 1

    # 2. Add edges.
    seen_edges: set[tuple[str, str, str]] = set()

    def add_once(u: str, v: str, edge_type: str, **attr: Any) -> bool:
        key = (u, v, edge_type)
        if key not in seen_edges:
            seen_edges.add(key)
            G.add_edge(u, v, edge_type=edge_type, **attr)
            return True
        return False

    for post in posts:
        pid = _post_node_id(post)
        aid = _author_node_id(post)
        plid = _platform_node_id(post.platform)

        if add_once(aid, pid, "AUTHORED"):
            stats.authored_edges += 1
        if add_once(pid, plid, "ON_PLATFORM"):
            stats.on_platform_edges += 1

        # Shared-content edges: Post -> original Post (if shared_from is known).
        if post.shared_from:
            target_hash = post.shared_from
            for target_post in content_to_posts.get(target_hash, []):
                target_pid = _post_node_id(target_post)
                if add_once(pid, target_pid, "SHARED_CONTENT"):
                    stats.shared_content_edges += 1

    # 3. Cross-platform mirror edges: posts with same content_hash on different platforms.
    for ch, same_content_posts in content_to_posts.items():
        platforms_seen = {p.platform for p in same_content_posts}
        if len(platforms_seen) > 1:
            # Add mirror edges between all pairs (undirected, so both directions).
            for i, p1 in enumerate(same_content_posts):
                for p2 in same_content_posts[i + 1 :]:
                    if p1.platform != p2.platform:
                        u = _post_node_id(p1)
                        v = _post_node_id(p2)
                        if add_once(u, v, "CROSS_PLATFORM_MIRROR"):
                            stats.cross_platform_mirror_edges += 1
                        if add_once(v, u, "CROSS_PLATFORM_MIRROR"):
                            stats.cross_platform_mirror_edges += 1

    return G, stats
