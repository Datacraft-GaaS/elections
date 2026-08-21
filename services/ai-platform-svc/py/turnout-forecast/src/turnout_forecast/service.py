from __future__ import annotations

import logging
from datetime import datetime, timezone

from fastapi import FastAPI, HTTPException

from turnout_forecast.forecaster import check_variance, clear_state, forecast, get_alerts, ingest
from turnout_forecast.views import TurnoutForecast, TurnoutIngest, VarianceAlert

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-turnout-forecast", version="0.1.0")


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _log_ingest(station_code: str) -> None:
    logger.info("eel.turnout.ingest.v1 station=%s", station_code)


def _log_forecast(station_code: str, pct: float) -> None:
    logger.info("eel.turnout.forecast.v1 station=%s pct=%.2f", station_code, pct)


@app.get("/v1/forecast/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


@app.post("/v1/forecast/ingest", status_code=204)
async def ingest_point(point: TurnoutIngest) -> None:
    await ingest(point)
    _log_ingest(point.station_code)


@app.post("/v1/forecast/station/{station_code}")
async def get_forecast(station_code: str, point: TurnoutIngest) -> TurnoutForecast:
    if point.station_code != station_code:
        raise HTTPException(status_code=400, detail="station_code mismatch")
    fc = await forecast(
        station_code=station_code,
        current_hour=point.hour,
        registered_voters=point.registered_voters,
        votes_cast=point.votes_cast_so_far,
    )
    _log_forecast(station_code, fc.forecasted_turnout_pct)
    return fc


@app.get("/v1/forecast/alerts")
async def list_alerts() -> list[VarianceAlert]:
    return get_alerts()
