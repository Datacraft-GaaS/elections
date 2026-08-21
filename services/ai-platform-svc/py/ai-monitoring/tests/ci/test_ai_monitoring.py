from __future__ import annotations

import asyncio

from httpx import ASGITransport, AsyncClient

from ai_monitoring.service import app, reports_store


BASE = "http://test"


def _client() -> AsyncClient:
    return AsyncClient(transport=ASGITransport(app=app), base_url=BASE)


def setup_function(_):
    reports_store.clear()


def test_health_ok():
    async def _run():
        async with _client() as c:
            r = await c.get("/v1/ai/monitoring/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_create_and_publish_report():
    async def _run():
        payload = {
            "report_week": "2027-W01",
            "layer_summaries": {"KIT": {"count": 42}},
            "pii_strip_stats": {"total": 10},
        }
        async with _client() as c:
            r_create = await c.post("/v1/ai/monitoring/reports", json=payload)
        assert r_create.status_code == 201
        body = r_create.json()
        assert body["status"] == "PENDING"
        assert body["published_at"] is None
        report_id = body["id"]

        async with _client() as c:
            r_pub = await c.post(f"/v1/ai/monitoring/reports/{report_id}/publish")
        assert r_pub.status_code == 200
        pub_body = r_pub.json()
        assert pub_body["status"] == "PUBLISHED"
        assert pub_body["published_at"] is not None

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_get_report():
    async def _run():
        payload = {
            "report_week": "2027-W02",
            "layer_summaries": {},
            "pii_strip_stats": {},
        }
        async with _client() as c:
            r = await c.post("/v1/ai/monitoring/reports", json=payload)
        report_id = r.json()["id"]

        async with _client() as c:
            r_get = await c.get(f"/v1/ai/monitoring/reports/{report_id}")
        assert r_get.status_code == 200
        assert r_get.json()["id"] == report_id

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_decision_summary_empty():
    async def _run():
        async with _client() as c:
            r = await c.get("/v1/ai/monitoring/decision-summary")
        assert r.status_code == 200
        assert r.json() == []

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_pii_stats_zeros():
    async def _run():
        async with _client() as c:
            r = await c.get("/v1/ai/monitoring/pii-stats")
        assert r.status_code == 200
        body = r.json()
        assert body["total_calls"] == 0
        assert body["bypass_count"] == 0

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())
