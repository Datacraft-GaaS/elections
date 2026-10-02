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

import logging
from collections import deque
from datetime import datetime, timezone

from fastapi import FastAPI
from uuid6 import uuid7

from anomaly.layers import detect_all
from anomaly.views import AlertBatch, AnomalyAlert, TelemetryPoint

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-anomaly", version="0.1.0")

_recent_alerts: deque[AnomalyAlert] = deque(maxlen=100)


def _uuid7str() -> str:
    return str(uuid7())


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _log_alert(alert: AnomalyAlert) -> None:
    logger.info(
        "eel.anomaly.alert.v1 alert_id=%s station=%s layer=%s severity=%s",
        alert.alert_id,
        alert.station_code,
        alert.layer,
        alert.severity,
    )


@app.get("/v1/anomaly/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


@app.post("/v1/anomaly/detect", status_code=200)
async def detect(points: list[TelemetryPoint]) -> AlertBatch:
    alerts = await detect_all(points)
    for alert in alerts:
        _log_alert(alert)
        _recent_alerts.append(alert)
    return AlertBatch(
        alerts=alerts,
        batch_id=_uuid7str(),
        generated_at=_now(),
    )


@app.get("/v1/anomaly/alerts")
async def list_alerts() -> list[AnomalyAlert]:
    return list(_recent_alerts)
