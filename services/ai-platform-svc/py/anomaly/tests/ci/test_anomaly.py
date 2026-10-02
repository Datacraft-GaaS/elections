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

import httpx
import pytest
from fastapi.testclient import TestClient

from anomaly.service import app
from anomaly.views import TelemetryPoint
from anomaly import layers as _layers


def _point(
    station_code: str = "STN001",
    votes_cast: int = 100,
    ballot_papers_issued: int = 100,
    spoilt_ballots: int = 2,
    kit_id: str = "KIT001",
) -> TelemetryPoint:
    return TelemetryPoint(
        station_code=station_code,
        timestamp=datetime.now(timezone.utc),
        votes_cast=votes_cast,
        ballot_papers_issued=ballot_papers_issued,
        spoilt_ballots=spoilt_ballots,
        kit_id=kit_id,
    )


def test_health() -> None:
    with TestClient(app) as client:
        r = client.get("/v1/anomaly/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}


def test_benford_layer() -> None:
    # All values start with 9 -- strongly violates Benford
    loop = asyncio.get_event_loop()
    points = [
        _point(station_code=f"STN{i:03d}", votes_cast=900 + i, ballot_papers_issued=900 + i)
        for i in range(20)
    ]
    alerts = loop.run_until_complete(_layers.layer3_benford(points))
    assert any(a.layer == "layer3-benford" for a in alerts), "Benford layer should fire"


def test_canary_layer() -> None:
    loop = asyncio.get_event_loop()
    points = [_point(station_code="CANARY_TEST")]
    alerts = loop.run_until_complete(_layers.layer5_canary(points))
    assert len(alerts) == 1
    assert alerts[0].layer == "layer5-canary"
    assert alerts[0].severity == "critical"


def test_cross_source() -> None:
    loop = asyncio.get_event_loop()
    # votes_cast >> ballot_papers_issued
    points = [_point(votes_cast=9000, ballot_papers_issued=100)]
    alerts = loop.run_until_complete(_layers.layer2_cross_source(points))
    assert len(alerts) == 1
    assert alerts[0].layer == "layer2-cross-source"


def test_isoforest_layer() -> None:
    loop = asyncio.get_event_loop()
    # Extreme outlier should be flagged
    points = [_point(votes_cast=999_999, ballot_papers_issued=1, spoilt_ballots=50_000)]
    alerts = loop.run_until_complete(_layers.layer1_isoforest(points))
    assert len(alerts) >= 1
    assert all(a.layer == "layer1-isoforest" for a in alerts)


def test_rate_limit() -> None:
    loop = asyncio.get_event_loop()
    # Reset the rate-limit dict so this test is independent
    _layers._last_alert_ts.clear()

    alert1 = _point(votes_cast=9000, ballot_papers_issued=100)
    raw1 = loop.run_until_complete(_layers.layer2_cross_source([alert1]))
    assert len(raw1) == 1

    deduped1 = loop.run_until_complete(_layers.layer7_dedup_ratelimit(raw1))
    assert len(deduped1) == 1

    # Same station again immediately -- should be suppressed
    raw2 = loop.run_until_complete(_layers.layer2_cross_source([alert1]))
    assert len(raw2) == 1
    deduped2 = loop.run_until_complete(_layers.layer7_dedup_ratelimit(raw2))
    assert len(deduped2) == 0, "Second alert within 60s should be deduplicated"
