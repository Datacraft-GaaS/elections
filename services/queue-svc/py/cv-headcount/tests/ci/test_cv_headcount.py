from __future__ import annotations

import asyncio
from datetime import datetime, timezone

from httpx import ASGITransport, AsyncClient

from cv_headcount.service import app

BASE = "http://test"


def _client() -> AsyncClient:
    return AsyncClient(transport=ASGITransport(app=app), base_url=BASE)


def test_health_ok() -> None:
    async def _run() -> None:
        async with _client() as c:
            r = await c.get("/v1/cv/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_headcount_fixture_source() -> None:
    async def _run() -> None:
        payload = {
            "station_code": "KEN-001",
            "image_source": "fixture://dummy",
            "captured_at": datetime.now(timezone.utc).isoformat(),
        }
        async with _client() as c:
            r = await c.post("/v1/cv/headcount", json=payload)
        assert r.status_code == 200
        body = r.json()
        assert body["station_code"] == "KEN-001"
        assert isinstance(body["headcount"], int)
        assert body["headcount"] > 0
        assert body["source"] == "fixture://dummy"

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_headcount_confidence_in_range() -> None:
    async def _run() -> None:
        payload = {
            "station_code": "KEN-002",
            "image_source": "fixture://dummy",
            "captured_at": datetime.now(timezone.utc).isoformat(),
        }
        async with _client() as c:
            r = await c.post("/v1/cv/headcount", json=payload)
        assert r.status_code == 200
        body = r.json()
        confidence = body["confidence"]
        assert 0.0 <= confidence <= 1.0

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_headcount_stub_non_fixture() -> None:
    async def _run() -> None:
        payload = {
            "station_code": "KEN-003",
            "image_source": "rtsp://camera.local/stream",
            "captured_at": datetime.now(timezone.utc).isoformat(),
        }
        async with _client() as c:
            r = await c.post("/v1/cv/headcount", json=payload)
        assert r.status_code == 200
        body = r.json()
        assert body["headcount"] == 0
        assert body["confidence"] == 0.0

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_headcount_result_has_result_id() -> None:
    async def _run() -> None:
        payload = {
            "station_code": "KEN-004",
            "image_source": "fixture://dummy",
            "captured_at": datetime.now(timezone.utc).isoformat(),
        }
        async with _client() as c:
            r = await c.post("/v1/cv/headcount", json=payload)
        assert r.status_code == 200
        body = r.json()
        assert "result_id" in body
        assert len(body["result_id"]) > 0
        assert "processed_at" in body

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())
