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

"""
Tests for AnomalyDetector: z-score logic, alert fields, severity,
cooldown, and pubsub NOTIFY.

Run: uv run pytest -vxs tests/ci
Async tests use asyncio.get_event_loop() per project convention.
"""

from __future__ import annotations

import asyncio
import importlib.util
import json
import sys as _sys
from pathlib import Path
from typing import Any
from unittest.mock import MagicMock

import pytest

# ---------------------------------------------------------------------------
# Import modules under test (filenames have dashes)
# ---------------------------------------------------------------------------

_BL_PATH = Path(__file__).parent.parent / "baseline-learner.py"
_BL_SPEC = importlib.util.spec_from_file_location("baseline_learner", _BL_PATH)
_bl_mod = importlib.util.module_from_spec(_BL_SPEC)  # type: ignore[arg-type]
_BL_SPEC.loader.exec_module(_bl_mod)  # type: ignore[union-attr]

_sys.modules["baseline_learner"] = _bl_mod

_AD_PATH = Path(__file__).parent.parent / "anomaly-detector.py"
_AD_SPEC = importlib.util.spec_from_file_location("anomaly_detector", _AD_PATH)
_ad_mod = importlib.util.module_from_spec(_AD_SPEC)  # type: ignore[arg-type]
_AD_SPEC.loader.exec_module(_ad_mod)  # type: ignore[union-attr]

AnomalyDetector = _ad_mod.AnomalyDetector
Alert = _ad_mod.Alert
FeatureVector = _bl_mod.FeatureVector
EntityType = _bl_mod.EntityType


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _cfg() -> dict[str, Any]:
    return {
        "baseline_window_days": 14,
        "zscore_threshold": 3.0,
        "entity_types": ["user", "service_account", "kit", "pod"],
        "loki": {"url": "http://loki:3100", "query_interval_seconds": 30},
        "postgres": {"dsn": "postgresql://test/test"},
        "ecc_pubsub": {"channel": "eel.ecc.ueba_alert.v1"},
        "alert_cooldown_seconds": 300,
        "features": {
            "login_hour_of_day": True,
            "query_rate_per_minute": True,
            "syscall_entropy": True,
            "bytes_egressed": True,
            "distinct_tables_accessed": True,
        },
    }


def _make_fv(
    entity_id: str = "user-1",
    entity_type: EntityType = EntityType.USER,
    hour_of_day: float = 10.0,
    query_rate: float = 5.0,
    syscall_entropy: float = 1.5,
    bytes_egressed: float = 1024.0,
    distinct_tables: float = 2.0,
) -> FeatureVector:
    return FeatureVector(
        entity_id=entity_id,
        entity_type=entity_type,
        hour_of_day=hour_of_day,
        query_rate_per_minute=query_rate,
        syscall_entropy=syscall_entropy,
        bytes_egressed=bytes_egressed,
        distinct_tables_accessed=distinct_tables,
    )


def _detector_with_baseline(
    baseline: dict[str, tuple[float, float]],
    cooldown: int = 300,
) -> AnomalyDetector:
    cfg = _cfg()
    cfg["alert_cooldown_seconds"] = cooldown
    detector = AnomalyDetector(cfg)
    notify_calls: list[Alert] = []

    async def fake_fetch(entity_id, entity_type):
        return baseline

    async def fake_notify(alert):
        notify_calls.append(alert)

    detector._fetch_baseline = fake_fetch  # type: ignore[method-assign]
    detector._notify = fake_notify  # type: ignore[method-assign]
    detector._pool = MagicMock()
    detector._notify_calls = notify_calls  # expose for assertions
    return detector


# ---------------------------------------------------------------------------
# 1. z-score below threshold: no alert
# ---------------------------------------------------------------------------


def test_no_alert_below_threshold() -> None:
    # mean=5, std=1 -> z = (5.5 - 5) / 1 = 0.5, well below 3.0
    baseline = {
        feat: (5.0, 1.0)
        for feat in FeatureVector.__dataclass_fields__
        if feat not in ("entity_id", "entity_type", "timestamp")
    }
    detector = _detector_with_baseline(baseline)
    fv = _make_fv(query_rate=5.5)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert alerts == []


# ---------------------------------------------------------------------------
# 2. z-score above threshold on one feature: alert with correct fields
# ---------------------------------------------------------------------------


def test_alert_above_threshold_one_feature() -> None:
    # query_rate z = (50 - 5) / 1 = 45 >> 3.0; others normal
    baseline = {
        "hour_of_day": (10.0, 1.0),
        "query_rate_per_minute": (5.0, 1.0),
        "syscall_entropy": (1.5, 0.5),
        "bytes_egressed": (1024.0, 100.0),
        "distinct_tables_accessed": (2.0, 0.5),
    }
    detector = _detector_with_baseline(baseline)
    fv = _make_fv(query_rate=50.0)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert len(alerts) == 1
    alert = alerts[0]
    assert "query_rate_per_minute" in alert.features_breached
    assert alert.entity_id == "user-1"
    assert alert.entity_type == EntityType.USER
    assert "query_rate_per_minute" in alert.z_scores
    assert alert.z_scores["query_rate_per_minute"] == pytest.approx(45.0)


# ---------------------------------------------------------------------------
# 3. Multiple features breached: single alert listing all
# ---------------------------------------------------------------------------


def test_single_alert_lists_all_breached_features() -> None:
    baseline = {
        "hour_of_day": (10.0, 0.5),
        "query_rate_per_minute": (5.0, 1.0),
        "syscall_entropy": (1.5, 0.1),
        "bytes_egressed": (1024.0, 50.0),
        "distinct_tables_accessed": (2.0, 0.3),
    }
    detector = _detector_with_baseline(baseline)
    # Drive query_rate and bytes_egressed both above threshold
    fv = _make_fv(query_rate=50.0, bytes_egressed=10000.0)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert len(alerts) == 1
    breached = set(alerts[0].features_breached)
    assert "query_rate_per_minute" in breached
    assert "bytes_egressed" in breached


# ---------------------------------------------------------------------------
# 4. Zero std: feature treated as no-drift (None z, not raised)
# ---------------------------------------------------------------------------


def test_zero_std_no_false_alert() -> None:
    baseline = {
        "hour_of_day": (10.0, 0.0),  # std=0 -> skip
        "query_rate_per_minute": (5.0, 0.0),
        "syscall_entropy": (1.5, 0.0),
        "bytes_egressed": (1024.0, 0.0),
        "distinct_tables_accessed": (2.0, 0.0),
    }
    detector = _detector_with_baseline(baseline)
    # All values wildly different from mean but std=0 -> no alert
    fv = _make_fv(query_rate=9999.0, bytes_egressed=999999.0)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert alerts == []


# ---------------------------------------------------------------------------
# 5. Severity escalation: z > 5 -> CRITICAL, z > 3 -> WARNING
# ---------------------------------------------------------------------------


def test_severity_warning_for_z_above_3() -> None:
    baseline = {
        "query_rate_per_minute": (5.0, 1.0),
        **{
            f: (0.0, 1.0)
            for f in [
                "hour_of_day",
                "syscall_entropy",
                "bytes_egressed",
                "distinct_tables_accessed",
            ]
        },
    }
    detector = _detector_with_baseline(baseline)
    # z = (8.5 - 5) / 1 = 3.5 -> WARNING
    fv = _make_fv(query_rate=8.5)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert len(alerts) == 1
    assert alerts[0].severity == "WARNING"


def test_severity_critical_for_z_above_5() -> None:
    baseline = {
        "query_rate_per_minute": (5.0, 1.0),
        **{
            f: (0.0, 1.0)
            for f in [
                "hour_of_day",
                "syscall_entropy",
                "bytes_egressed",
                "distinct_tables_accessed",
            ]
        },
    }
    detector = _detector_with_baseline(baseline)
    # z = (11 - 5) / 1 = 6.0 -> CRITICAL
    fv = _make_fv(query_rate=11.0)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert len(alerts) == 1
    assert alerts[0].severity == "CRITICAL"


# ---------------------------------------------------------------------------
# 6. Alert cooldown prevents duplicate alerts
# ---------------------------------------------------------------------------


def test_cooldown_suppresses_second_alert() -> None:
    baseline = {
        "query_rate_per_minute": (5.0, 1.0),
        **{
            f: (0.0, 1.0)
            for f in [
                "hour_of_day",
                "syscall_entropy",
                "bytes_egressed",
                "distinct_tables_accessed",
            ]
        },
    }
    # Very long cooldown
    detector = _detector_with_baseline(baseline, cooldown=9999)
    fv = _make_fv(query_rate=50.0)
    loop = asyncio.get_event_loop()
    first = loop.run_until_complete(detector.detect(fv))
    second = loop.run_until_complete(detector.detect(fv))
    assert len(first) == 1
    assert second == []


# ---------------------------------------------------------------------------
# 7. pubsub NOTIFY is called on alert
# ---------------------------------------------------------------------------


def test_notify_called_on_alert() -> None:
    baseline = {
        "query_rate_per_minute": (5.0, 1.0),
        **{
            f: (0.0, 1.0)
            for f in [
                "hour_of_day",
                "syscall_entropy",
                "bytes_egressed",
                "distinct_tables_accessed",
            ]
        },
    }
    detector = _detector_with_baseline(baseline, cooldown=0)
    fv = _make_fv(query_rate=50.0)
    loop = asyncio.get_event_loop()
    alerts = loop.run_until_complete(detector.detect(fv))
    assert len(alerts) == 1
    assert len(detector._notify_calls) == 1
    notified = detector._notify_calls[0]
    assert notified.entity_id == "user-1"
    assert "query_rate_per_minute" in notified.features_breached
    # Payload must be JSON-serialisable
    payload = json.loads(notified.to_json())
    assert payload["entity_id"] == "user-1"
    assert payload["severity"] in ("WARNING", "CRITICAL")
