from __future__ import annotations

import asyncio
from datetime import datetime, timezone

from httpx import ASGITransport, AsyncClient

from turnout_forecast.forecaster import _alerts, _station_data, check_variance
from turnout_forecast.service import app
from turnout_forecast.views import TurnoutForecast

BASE = "http://test"


def _client() -> AsyncClient:
    return AsyncClient(transport=ASGITransport(app=app), base_url=BASE)


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def setup_function(_: object) -> None:
    _station_data.clear()
    _alerts.clear()


def test_health() -> None:
    async def _run() -> None:
        async with _client() as c:
            r = await c.get("/v1/forecast/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_ingest_and_forecast() -> None:
    async def _run() -> None:
        station = "STN001"
        for hour in range(5):
            pt = {
                "station_code": station,
                "hour": hour * 3,
                "registered_voters": 1000,
                "votes_cast_so_far": int(1000 * (hour * 3 / 24) * 0.7),
                "recorded_at": _now_iso(),
            }
            async with _client() as c:
                r = await c.post("/v1/forecast/ingest", json=pt)
            assert r.status_code == 204

        req = {
            "station_code": station,
            "hour": 14,
            "registered_voters": 1000,
            "votes_cast_so_far": 450,
            "recorded_at": _now_iso(),
        }
        async with _client() as c:
            r = await c.post(f"/v1/forecast/station/{station}", json=req)
        assert r.status_code == 200
        body = r.json()
        assert 0.0 <= body["forecasted_turnout_pct"] <= 100.0
        assert body["station_code"] == station

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_variance_alert() -> None:
    async def _run() -> None:
        fc = TurnoutForecast(
            forecast_id="fc1",
            station_code="STN002",
            forecasted_turnout_pct=60.0,
            lower_bound_pct=55.0,
            upper_bound_pct=65.0,
            variance_alert=False,
            generated_at=datetime.now(timezone.utc),
        )
        alert = await check_variance("STN002", fc, 85.0)
        assert alert is not None
        assert alert.deviation_pct == 25.0

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_no_variance() -> None:
    async def _run() -> None:
        fc = TurnoutForecast(
            forecast_id="fc2",
            station_code="STN003",
            forecasted_turnout_pct=60.0,
            lower_bound_pct=55.0,
            upper_bound_pct=65.0,
            variance_alert=False,
            generated_at=datetime.now(timezone.utc),
        )
        alert = await check_variance("STN003", fc, 62.0)
        assert alert is None

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())
