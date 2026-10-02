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

"""FastAPI HTTP API for the cross-platform CIB detection service.

Routes:
  POST /cib/ingest/{platform}  -- ingest raw platform records
  POST /cib/detect             -- run correlation + GNN on current store
  GET  /cib/clusters           -- list detected mirror clusters
  GET  /cib/reports            -- structured report of current detection run
"""

from __future__ import annotations

import asyncio
from collections import defaultdict
from datetime import datetime, timezone
from typing import Annotated, Any
from uuid import uuid4

from fastapi import FastAPI, HTTPException, Path
from pydantic import BaseModel, ConfigDict

from .corr import CorrelatorResult, CrossPlatformCorrelator
from .graph import GraphStats, build_graph
from .loaders import (
    FacebookLoader,
    TelegramLoader,
    TikTokLoader,
    TwitterLoader,
    WhatsAppBroadcastLoader,
)
from .models import Post

# ---------------------------------------------------------------------------
# State (in-memory; replace with persistent store in production)
# ---------------------------------------------------------------------------

_store: list[Post] = []
_last_result: CorrelatorResult | None = None
_last_graph_stats: GraphStats | None = None

_LOADERS = {
    "facebook": FacebookLoader(),
    "twitter": TwitterLoader(),
    "tiktok": TikTokLoader(),
    "whatsapp_broadcast": WhatsAppBroadcastLoader(),
    "telegram": TelegramLoader(),
}

# ---------------------------------------------------------------------------
# Request / response models
# ---------------------------------------------------------------------------


class IngestRequest(BaseModel):
    model_config = ConfigDict(extra="allow")
    records: list[dict[str, Any]]


class IngestResponse(BaseModel):
    platform: str
    ingested: int
    total_store_size: int


class DetectRequest(BaseModel):
    window_minutes: float = 60.0
    min_platforms: int = 2


class DetectResponse(BaseModel):
    total_posts_analysed: int
    mirror_clusters_found: int
    flagged_content_hashes: list[str]
    graph_stats: dict[str, int]


class ClusterSummary(BaseModel):
    content_hash: str
    platforms: list[str]
    post_count: int
    window_minutes: float | None


class ClustersResponse(BaseModel):
    clusters: list[ClusterSummary]


class ReportEntry(BaseModel):
    content_hash: str
    platforms: list[str]
    post_count: int
    earliest_ts: str | None
    latest_ts: str | None
    window_minutes: float | None


class ReportResponse(BaseModel):
    generated_at: str
    total_posts_analysed: int
    report_id: str
    entries: list[ReportEntry]


# ---------------------------------------------------------------------------
# App
# ---------------------------------------------------------------------------

app = FastAPI(
    title="eel-cross-platform-cib",
    description="Cross-platform CIB detection: GraphSAGE + content correlation",
    version="0.1.0",
)


@app.post("/cib/ingest/{platform}", response_model=IngestResponse)
async def ingest(
    platform: Annotated[
        str,
        Path(
            description="Platform name (facebook|twitter|tiktok|whatsapp_broadcast|telegram)"
        ),
    ],
    body: IngestRequest,
) -> IngestResponse:
    """Ingest raw platform records and unify into the Post schema."""
    loader = _LOADERS.get(platform)
    if loader is None:
        raise HTTPException(status_code=422, detail=f"Unknown platform: {platform!r}")
    posts = await asyncio.get_event_loop().run_in_executor(
        None, loader.load_batch, body.records
    )
    _store.extend(posts)
    return IngestResponse(
        platform=platform, ingested=len(posts), total_store_size=len(_store)
    )


@app.post("/cib/detect", response_model=DetectResponse)
async def detect(body: DetectRequest) -> DetectResponse:
    """Run cross-platform correlation on the current post store."""
    global _last_result, _last_graph_stats

    correlator = CrossPlatformCorrelator(
        window_minutes=body.window_minutes,
        min_platforms=body.min_platforms,
    )
    result = await asyncio.get_event_loop().run_in_executor(
        None, correlator.correlate, list(_store)
    )
    _last_result = result

    _, stats = await asyncio.get_event_loop().run_in_executor(
        None, build_graph, list(_store)
    )
    _last_graph_stats = stats

    return DetectResponse(
        total_posts_analysed=result.total_posts_analysed,
        mirror_clusters_found=len(result.mirror_clusters),
        flagged_content_hashes=sorted(result.flagged_post_hashes),
        graph_stats={
            "post_nodes": stats.post_nodes,
            "author_nodes": stats.author_nodes,
            "platform_nodes": stats.platform_nodes,
            "authored_edges": stats.authored_edges,
            "on_platform_edges": stats.on_platform_edges,
            "shared_content_edges": stats.shared_content_edges,
            "cross_platform_mirror_edges": stats.cross_platform_mirror_edges,
        },
    )


@app.get("/cib/clusters", response_model=ClustersResponse)
async def clusters() -> ClustersResponse:
    """List all detected mirror clusters from the last detection run."""
    if _last_result is None:
        return ClustersResponse(clusters=[])
    summaries = [
        ClusterSummary(
            content_hash=c.content_hash,
            platforms=sorted(c.platforms),
            post_count=len(c.posts),
            window_minutes=c.window_minutes(),
        )
        for c in _last_result.mirror_clusters
        if c.is_cross_platform
    ]
    return ClustersResponse(clusters=summaries)


@app.get("/cib/reports", response_model=ReportResponse)
async def reports() -> ReportResponse:
    """Full structured report for the last detection run."""
    if _last_result is None:
        return ReportResponse(
            generated_at=datetime.now(tz=timezone.utc).isoformat(),
            total_posts_analysed=0,
            report_id=str(uuid4()),
            entries=[],
        )
    entries = [
        ReportEntry(
            content_hash=c.content_hash,
            platforms=sorted(c.platforms),
            post_count=len(c.posts),
            earliest_ts=c.earliest_ts.isoformat() if c.earliest_ts else None,
            latest_ts=c.latest_ts.isoformat() if c.latest_ts else None,
            window_minutes=c.window_minutes(),
        )
        for c in _last_result.mirror_clusters
        if c.is_cross_platform
    ]
    return ReportResponse(
        generated_at=datetime.now(tz=timezone.utc).isoformat(),
        total_posts_analysed=_last_result.total_posts_analysed,
        report_id=str(uuid4()),
        entries=entries,
    )
