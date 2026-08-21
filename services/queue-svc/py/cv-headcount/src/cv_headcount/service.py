from __future__ import annotations

import logging
from datetime import datetime, timezone

from fastapi import FastAPI
from uuid6 import uuid7

from cv_headcount.detector import estimate_headcount
from cv_headcount.views import HeadcountRequest, HeadcountResult

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-cv-headcount", version="0.1.0")


def uuid7str() -> str:
    return str(uuid7())


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _log_headcount(result_id: str, station_code: str, headcount: int) -> None:
    logger.info(
        "eel.cv.headcount.v1 result_id=%s station=%s count=%d",
        result_id,
        station_code,
        headcount,
    )


@app.get("/v1/cv/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


@app.post("/v1/cv/headcount", response_model=HeadcountResult, status_code=200)
async def headcount(body: HeadcountRequest) -> HeadcountResult:
    count, confidence = await estimate_headcount(body.station_code, body.image_source)
    result_id = uuid7str()
    result = HeadcountResult(
        result_id=result_id,
        station_code=body.station_code,
        headcount=count,
        confidence=confidence,
        source=body.image_source,
        processed_at=_now(),
    )
    _log_headcount(result_id, body.station_code, count)
    return result
