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

from __future__ import annotations

import asyncio
from datetime import datetime, timezone

from httpx import ASGITransport, AsyncClient

from speech_integrity.service import app, flags_store

BASE = "http://test"


def _client() -> AsyncClient:
    return AsyncClient(transport=ASGITransport(app=app), base_url=BASE)


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def setup_function(_: object) -> None:
    flags_store.clear()


def test_health() -> None:
    async def _run() -> None:
        async with _client() as c:
            r = await c.get("/v1/speech/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_classify_flags_misinformation() -> None:
    async def _run() -> None:
        items = [
            {
                "item_id": "i1",
                "platform": "twitter",
                "content_text": "This is a hoax and a lie spread everywhere",
                "author_id": "a1",
                "published_at": _now_iso(),
                "media_urls": [],
            }
        ]
        async with _client() as c:
            r = await c.post("/v1/speech/classify", json=items)
        assert r.status_code == 200
        flags = r.json()
        assert len(flags) >= 1
        types = {f["flag_type"] for f in flags}
        assert "misinformation" in types

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_cib_clustering() -> None:
    async def _run() -> None:
        text = "a" * 40  # same length bucket
        items = [
            {
                "item_id": f"i{n}",
                "platform": "facebook",
                "content_text": text,
                "author_id": f"author{n}",
                "published_at": _now_iso(),
                "media_urls": [],
            }
            for n in range(3)
        ]
        async with _client() as c:
            r = await c.post("/v1/speech/cib-detect", json=items)
        assert r.status_code == 200
        clusters = r.json()
        assert len(clusters) >= 1
        assert len(clusters[0]["account_ids"]) >= 2

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_deepfake_stub() -> None:
    async def _run() -> None:
        item = {
            "item_id": "i99",
            "platform": "youtube",
            "content_text": "normal video",
            "author_id": "a99",
            "published_at": _now_iso(),
            "media_urls": ["https://cdn.example.com/video.mp4"],
        }
        async with _client() as c:
            r = await c.post("/v1/speech/deepfake", json=item)
        assert r.status_code == 200
        body = r.json()
        assert body["is_deepfake"] is False

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_analyst_decision() -> None:
    async def _run() -> None:
        items = [
            {
                "item_id": "i2",
                "platform": "twitter",
                "content_text": "fake news hoax fabricated story",
                "author_id": "a2",
                "published_at": _now_iso(),
                "media_urls": [],
            }
        ]
        async with _client() as c:
            r = await c.post("/v1/speech/classify", json=items)
        assert r.status_code == 200
        flags = r.json()
        assert len(flags) >= 1
        flag_id = flags[0]["flag_id"]

        decision = {"flag_id": flag_id, "decision": "confirmed", "decision_by": "analyst_1"}
        async with _client() as c:
            r2 = await c.patch(f"/v1/speech/flags/{flag_id}/decision", json=decision)
        assert r2.status_code == 200
        body = r2.json()
        assert body["analyst_decision"] == "confirmed"
        assert body["decision_by"] == "analyst_1"
        assert body["decision_at"] is not None

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())
