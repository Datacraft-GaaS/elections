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


def test_decision_summary_declares_unimplemented():
    """501, not a 200 with an empty list.

    The route cannot be served from this service's state: ModelDecisionSummary
    is per-model and a WeeklyDetectionReport carries no model identity. It
    previously returned [] for an empty store and raised ValidationError -- a
    500 -- for a store with a published report, which is the state a caller is
    actually in. Asserting the 501 keeps the gap declared.
    """

    async def _run():
        async with _client() as c:
            r = await c.get("/v1/ai/monitoring/decision-summary")
        assert r.status_code == 501, r.text

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_decision_summary_with_published_report_is_declared_not_broken():
    """The canary for the defect above.

    The old body only entered its loop for a PUBLISHED report, so the empty
    case passed while every populated call raised ValidationError and returned
    500. Publishing first and asserting 501 -- declared unimplemented -- rather
    than 500 is what separates "this route says what it cannot do" from "this
    route breaks the moment it is given data". Asserting merely `< 500` would
    accept the 500 the old body produced.
    """

    async def _run():
        async with _client() as c:
            created = await c.post(
                "/v1/ai/monitoring/reports",
                json={
                    "report_week": "2026-W42",
                    "layer_summaries": {"KIT": {"count": 42}},
                    "pii_strip_stats": {},
                },
            )
            report_id = created.json()["id"]
            await c.post(f"/v1/ai/monitoring/reports/{report_id}/publish")
            r = await c.get("/v1/ai/monitoring/decision-summary")
        assert r.status_code == 501, f"expected a declared 501, got {r.status_code}: {r.text}"

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
