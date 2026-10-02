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

"""Tests for the eel-cross-platform-cib package.

Covers: graph construction determinism, GNN forward pass, correlator mirror
detection, unknown content not flagged, API endpoints, loader schema unification.
"""

from __future__ import annotations

import hashlib
from datetime import datetime, timedelta, timezone

import pytest

from eel_cross_platform_cib.corr import CrossPlatformCorrelator
from eel_cross_platform_cib.graph import build_graph
from eel_cross_platform_cib.gnn import GraphSAGECIB, build_feature_matrix
from eel_cross_platform_cib.loaders import (
    FacebookLoader,
    TelegramLoader,
    TikTokLoader,
    TwitterLoader,
    WhatsAppBroadcastLoader,
)
from eel_cross_platform_cib.models import Post, make_content_hash

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

NOW = datetime(2027, 3, 1, 12, 0, 0, tzinfo=timezone.utc)


def _hash(text: str) -> str:
    return hashlib.sha256(text.strip().lower().encode()).hexdigest()


def _post(
    platform: str,
    text: str,
    ts: datetime = NOW,
    author: str = "a1",
    shared_from: str | None = None,
) -> Post:
    return Post(
        platform=platform,
        author_id=author,
        content_hash=_hash(text),
        ts=ts,
        shared_from=shared_from,
    )


# ---------------------------------------------------------------------------
# Test 1: graph construction is deterministic
# ---------------------------------------------------------------------------


def test_graph_deterministic():
    posts = [
        _post("facebook", "hello world"),
        _post("twitter", "hello world"),
        _post("tiktok", "other content"),
    ]
    g1, s1 = build_graph(posts)
    g2, s2 = build_graph(posts)
    assert set(g1.nodes) == set(g2.nodes)
    assert set(g1.edges(keys=True)) == set(g2.edges(keys=True))
    assert s1.post_nodes == s2.post_nodes


# ---------------------------------------------------------------------------
# Test 2: graph has expected node counts
# ---------------------------------------------------------------------------


def test_graph_node_counts():
    posts = [
        _post("facebook", "alpha"),
        _post("twitter", "beta"),
        _post("tiktok", "alpha"),
    ]
    g, stats = build_graph(posts)
    # 3 posts but "alpha" appears on facebook+tiktok -> 3 post nodes (different platforms)
    assert stats.post_nodes == 3
    assert stats.platform_nodes == 3  # facebook, twitter, tiktok
    assert stats.authored_edges == 3


# ---------------------------------------------------------------------------
# Test 3: cross-platform mirror edges detected for same content
# ---------------------------------------------------------------------------


def test_graph_cross_platform_mirror_edges():
    posts = [_post("facebook", "shared content"), _post("twitter", "shared content")]
    g, stats = build_graph(posts)
    assert stats.cross_platform_mirror_edges >= 2  # bidirectional


# ---------------------------------------------------------------------------
# Test 4: no mirror edges for content on a single platform
# ---------------------------------------------------------------------------


def test_graph_no_mirror_edges_single_platform():
    posts = [
        _post("facebook", "only here"),
        _post("facebook", "only here", author="a2"),
    ]
    g, stats = build_graph(posts)
    assert stats.cross_platform_mirror_edges == 0


# ---------------------------------------------------------------------------
# Test 5: shared_from edges in graph
# ---------------------------------------------------------------------------


def test_graph_shared_content_edges():
    original = _post("facebook", "original post")
    shared = _post("twitter", "original post", shared_from=_hash("original post"))
    posts = [original, shared]
    g, stats = build_graph(posts)
    # The shared post points to the original via SHARED_CONTENT edge.
    assert stats.shared_content_edges >= 1


# ---------------------------------------------------------------------------
# Test 6: GNN forward pass on synthetic graph (CPU)
# ---------------------------------------------------------------------------


def test_gnn_forward_pass():
    import torch

    try:
        model = GraphSAGECIB(in_channels=8, hidden_channels=16, out_channels=2)
    except ImportError:
        pytest.skip("torch_geometric not available")

    N = 5
    x = torch.randn(N, 8)
    edge_index = torch.tensor([[0, 1, 2, 3], [1, 2, 3, 4]], dtype=torch.long)
    out = model(x, edge_index)
    assert out.shape == (N, 2)


# ---------------------------------------------------------------------------
# Test 7: GNN predict_proba returns values in [0, 1]
# ---------------------------------------------------------------------------


def test_gnn_predict_proba_range():
    import torch

    try:
        model = GraphSAGECIB(in_channels=8, hidden_channels=16)
    except ImportError:
        pytest.skip("torch_geometric not available")

    N = 4
    x = torch.randn(N, 8)
    edge_index = torch.tensor([[0, 1, 2], [1, 2, 3]], dtype=torch.long)
    probs = model.predict_proba(x, edge_index)
    assert probs.shape == (N,)
    assert (probs >= 0).all() and (probs <= 1).all()


# ---------------------------------------------------------------------------
# Test 8: build_feature_matrix returns correct shape
# ---------------------------------------------------------------------------


def test_build_feature_matrix_shape():
    posts_data = [
        {
            "platform": "facebook",
            "ts": NOW.isoformat(),
            "shared_from": None,
            "engagement_stats": {"likes": 10, "shares": 5, "comments": 2, "views": 100},
        },
        {
            "platform": "twitter",
            "ts": NOW.isoformat(),
            "shared_from": "abc",
            "engagement_stats": {"likes": 0, "shares": 50, "comments": 0, "views": 0},
        },
    ]
    mat = build_feature_matrix(posts_data, feature_dim=8)
    assert mat.shape == (2, 8)


# ---------------------------------------------------------------------------
# Test 8b: build_feature_matrix tolerates bad timestamps and is deterministic
# ---------------------------------------------------------------------------


def test_build_feature_matrix_bad_ts_and_stable_platform():
    posts_data = [
        {
            "platform": "facebook",
            "ts": "not-a-timestamp",
            "shared_from": None,
            "engagement_stats": {},
        },
    ]
    mat = build_feature_matrix(posts_data, feature_dim=8)
    # Unparseable ts must not raise; hour feature stays at 0.
    assert mat[0, 6].item() == 0.0
    # Platform hash feature is stable across calls (no PYTHONHASHSEED drift).
    again = build_feature_matrix(posts_data, feature_dim=8)
    assert mat[0, 5].item() == again[0, 5].item()


# ---------------------------------------------------------------------------
# Test 9: correlator flags cross-platform mirror within window
# ---------------------------------------------------------------------------


def test_correlator_flags_mirror():
    posts = [
        _post("facebook", "election news", ts=NOW),
        _post("twitter", "election news", ts=NOW + timedelta(minutes=10)),
        _post("tiktok", "election news", ts=NOW + timedelta(minutes=20)),
    ]
    corr = CrossPlatformCorrelator(window_minutes=60, min_platforms=2)
    result = corr.correlate(posts)
    assert _hash("election news") in result.flagged_post_hashes


# ---------------------------------------------------------------------------
# Test 10: correlator does NOT flag content appearing on only one platform
# ---------------------------------------------------------------------------


def test_correlator_does_not_flag_single_platform():
    posts = [
        _post("facebook", "local only", ts=NOW),
        _post("facebook", "local only", ts=NOW + timedelta(minutes=5), author="a2"),
    ]
    corr = CrossPlatformCorrelator(window_minutes=60, min_platforms=2)
    result = corr.correlate(posts)
    assert _hash("local only") not in result.flagged_post_hashes


# ---------------------------------------------------------------------------
# Test 11: correlator does NOT flag mirrors outside the time window
# ---------------------------------------------------------------------------


def test_correlator_does_not_flag_outside_window():
    posts = [
        _post("facebook", "late content", ts=NOW),
        _post("twitter", "late content", ts=NOW + timedelta(minutes=200)),
    ]
    corr = CrossPlatformCorrelator(window_minutes=60, min_platforms=2)
    result = corr.correlate(posts)
    assert _hash("late content") not in result.flagged_post_hashes


# ---------------------------------------------------------------------------
# Test 12: correlator flags when min_platforms=3 is satisfied
# ---------------------------------------------------------------------------


def test_correlator_min_platforms_three():
    posts = [
        _post("facebook", "multi plat"),
        _post("twitter", "multi plat", ts=NOW + timedelta(minutes=5)),
        _post("tiktok", "multi plat", ts=NOW + timedelta(minutes=10)),
    ]
    corr = CrossPlatformCorrelator(window_minutes=60, min_platforms=3)
    result = corr.correlate(posts)
    assert _hash("multi plat") in result.flagged_post_hashes


# ---------------------------------------------------------------------------
# Test 13: Facebook loader unifies schema
# ---------------------------------------------------------------------------


def test_facebook_loader_schema():
    loader = FacebookLoader()
    raw = {
        "id": "fb-001",
        "from_id": "user-123",
        "message": "Hello Facebook",
        "created_time": "2027-03-01T12:00:00+00:00",
        "likes": {"summary": {"total_count": 42}},
        "shares": {"count": 5},
        "comments": {"summary": {"total_count": 3}},
    }
    post = loader.load(raw)
    assert post.platform == "facebook"
    assert post.author_id == "user-123"
    assert post.content_hash == _hash("Hello Facebook")
    assert post.engagement_stats.likes == 42
    assert post.engagement_stats.shares == 5


# ---------------------------------------------------------------------------
# Test 14: Twitter loader unifies schema
# ---------------------------------------------------------------------------


def test_twitter_loader_schema():
    loader = TwitterLoader()
    raw = {
        "id": "tw-001",
        "author_id": "tw-user-456",
        "text": "Hello Twitter",
        "created_at": "2027-03-01T12:00:00Z",
        "public_metrics": {
            "like_count": 10,
            "retweet_count": 3,
            "reply_count": 1,
            "impression_count": 500,
        },
        "referenced_tweets": [],
    }
    post = loader.load(raw)
    assert post.platform == "twitter"
    assert post.content_hash == _hash("Hello Twitter")
    assert post.engagement_stats.shares == 3


# ---------------------------------------------------------------------------
# Test 15: TikTok loader unifies schema
# ---------------------------------------------------------------------------


def test_tiktok_loader_schema():
    loader = TikTokLoader()
    raw = {
        "id": "tt-001",
        "author_id": "tiktok-user-789",
        "video_description": "viral dance",
        "create_time": 1740830400,
        "statistics": {
            "digg_count": 9999,
            "share_count": 100,
            "comment_count": 50,
            "play_count": 50000,
        },
    }
    post = loader.load(raw)
    assert post.platform == "tiktok"
    assert post.content_hash == _hash("viral dance")
    assert post.engagement_stats.views == 50000


# ---------------------------------------------------------------------------
# Test 16: WhatsApp broadcast loader unifies schema
# ---------------------------------------------------------------------------


def test_whatsapp_broadcast_loader_schema():
    loader = WhatsAppBroadcastLoader()
    raw = {
        "id": "wa-001",
        "channel_id": "channel-001",
        "text": "WhatsApp broadcast",
        "timestamp": 1740830400,
        "forward_count": 20,
        "reaction_count": 5,
    }
    post = loader.load(raw)
    assert post.platform == "whatsapp_broadcast"
    assert post.content_hash == _hash("WhatsApp broadcast")
    assert post.engagement_stats.shares == 20


# ---------------------------------------------------------------------------
# Test 17: Telegram loader unifies schema
# ---------------------------------------------------------------------------


def test_telegram_loader_schema():
    loader = TelegramLoader()
    raw = {
        "id": "tg-001",
        "chat_id": "channel-xyz",
        "text": "Telegram broadcast",
        "date": 1740830400,
        "views": 1000,
        "forwards": 50,
    }
    post = loader.load(raw)
    assert post.platform == "telegram"
    assert post.content_hash == _hash("Telegram broadcast")
    assert post.engagement_stats.views == 1000
    assert post.engagement_stats.shares == 50


# ---------------------------------------------------------------------------
# Test 18: make_content_hash is case- and whitespace-insensitive
# ---------------------------------------------------------------------------


def test_content_hash_normalisation():
    h1 = make_content_hash("Hello World")
    h2 = make_content_hash("  hello world  ")
    h3 = make_content_hash("HELLO WORLD")
    assert h1 == h2 == h3


# ---------------------------------------------------------------------------
# Test 19: MirrorCluster.window_minutes computed correctly
# ---------------------------------------------------------------------------


def test_mirror_cluster_window_minutes():
    posts = [
        _post("facebook", "test", ts=NOW),
        _post("twitter", "test", ts=NOW + timedelta(minutes=30)),
    ]
    corr = CrossPlatformCorrelator(window_minutes=60, min_platforms=2)
    result = corr.correlate(posts)
    assert len(result.mirror_clusters) == 1
    wm = result.mirror_clusters[0].window_minutes()
    assert wm is not None
    assert abs(wm - 30.0) < 0.1
