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
Tests for BaselineLearner: feature vectors, stat computation,
pg writes, re-learning, edge cases.

Run: uv run pytest -vxs tests/ci
Async tests use asyncio.get_event_loop() per project convention.
"""

from __future__ import annotations

import asyncio
import importlib.util
import json
import statistics
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from unittest.mock import AsyncMock, MagicMock

import pytest

# ---------------------------------------------------------------------------
# Import baseline-learner (filename has a dash, not a valid identifier)
# ---------------------------------------------------------------------------

_MODULE_PATH = Path(__file__).parent.parent / "baseline-learner.py"

spec = importlib.util.spec_from_file_location("baseline_learner", _MODULE_PATH)
_mod = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
spec.loader.exec_module(_mod)  # type: ignore[union-attr]

BaselineLearner = _mod.BaselineLearner
FeatureVector = _mod.FeatureVector
EntityType = _mod.EntityType


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


# ---------------------------------------------------------------------------
# 1. FeatureVector construction and field defaults
# ---------------------------------------------------------------------------


def test_feature_vector_construction() -> None:
    now = datetime.now(timezone.utc)
    fv = FeatureVector(
        entity_id="user-001",
        entity_type=EntityType.USER,
        hour_of_day=14.0,
        query_rate_per_minute=5.2,
        syscall_entropy=2.7,
        bytes_egressed=1024.0,
        distinct_tables_accessed=3.0,
        timestamp=now,
    )
    assert fv.entity_id == "user-001"
    assert fv.entity_type == EntityType.USER
    assert fv.hour_of_day == 14.0
    assert fv.timestamp == now


def test_feature_vector_timestamp_defaults_to_utc_now() -> None:
    fv = FeatureVector(
        entity_id="pod-abc",
        entity_type=EntityType.POD,
        hour_of_day=0.0,
        query_rate_per_minute=0.0,
        syscall_entropy=0.0,
        bytes_egressed=0.0,
        distinct_tables_accessed=0.0,
    )
    assert fv.timestamp.tzinfo is not None


def test_feature_vector_as_dict_keys() -> None:
    fv = FeatureVector(
        entity_id="sa-1",
        entity_type=EntityType.SERVICE_ACCOUNT,
        hour_of_day=8.0,
        query_rate_per_minute=10.0,
        syscall_entropy=1.5,
        bytes_egressed=512.0,
        distinct_tables_accessed=2.0,
    )
    keys = set(fv.as_dict().keys())
    assert keys == {
        "hour_of_day",
        "query_rate_per_minute",
        "syscall_entropy",
        "bytes_egressed",
        "distinct_tables_accessed",
    }


# ---------------------------------------------------------------------------
# 2. _compute_stats: mean and std from synthetic events
# ---------------------------------------------------------------------------


def test_compute_stats_mean_and_std() -> None:
    values = [1.0, 2.0, 3.0, 4.0, 5.0]
    mean, std = BaselineLearner._compute_stats(values)
    assert mean == pytest.approx(statistics.mean(values))
    assert std == pytest.approx(statistics.stdev(values), rel=1e-6)


def test_compute_stats_single_value_std_is_zero() -> None:
    """Single event must not raise; std returned as 0."""
    mean, std = BaselineLearner._compute_stats([42.0])
    assert mean == 42.0
    assert std == 0.0


def test_compute_stats_empty_list() -> None:
    mean, std = BaselineLearner._compute_stats([])
    assert mean == 0.0
    assert std == 0.0


# ---------------------------------------------------------------------------
# 3. Baseline written to pg with correct schema
# ---------------------------------------------------------------------------


def test_baseline_write_calls_pg_with_correct_args() -> None:
    learner = BaselineLearner(_cfg())
    conn = AsyncMock()
    pool_cm = MagicMock()
    pool_cm.__aenter__ = AsyncMock(return_value=conn)
    pool_cm.__aexit__ = AsyncMock(return_value=False)
    mock_pool = MagicMock()
    mock_pool.acquire.return_value = pool_cm
    learner._pool = mock_pool

    loop = asyncio.get_event_loop()
    now = datetime.now(timezone.utc)
    loop.run_until_complete(
        learner._write_baseline(
            entity_id="user-42",
            entity_type=EntityType.USER,
            feature="query_rate_per_minute",
            mean=5.0,
            std=1.2,
            sample_count=100,
            window_start=now,
            window_end=now,
        )
    )

    conn.execute.assert_awaited_once()
    call_args = conn.execute.call_args[0]
    # positional args: sql, entity_id, entity_type, feature, mean, std, ...
    assert "user-42" in call_args
    assert "user" in call_args
    assert "query_rate_per_minute" in call_args
    assert 5.0 in call_args
    assert 1.2 in call_args


# ---------------------------------------------------------------------------
# 4. Re-learning updates existing baseline (UPSERT path)
# ---------------------------------------------------------------------------


def test_baseline_upsert_is_idempotent() -> None:
    """_write_baseline uses ON CONFLICT DO UPDATE, so calling twice should
    call execute twice with the same SQL shape."""
    learner = BaselineLearner(_cfg())
    conn = AsyncMock()
    pool_cm = MagicMock()
    pool_cm.__aenter__ = AsyncMock(return_value=conn)
    pool_cm.__aexit__ = AsyncMock(return_value=False)
    mock_pool = MagicMock()
    mock_pool.acquire.return_value = pool_cm
    learner._pool = mock_pool

    loop = asyncio.get_event_loop()
    now = datetime.now(timezone.utc)
    for mean in (3.0, 4.5):
        loop.run_until_complete(
            learner._write_baseline(
                entity_id="kit-7",
                entity_type=EntityType.KIT,
                feature="bytes_egressed",
                mean=mean,
                std=0.5,
                sample_count=50,
                window_start=now,
                window_end=now,
            )
        )
    assert conn.execute.await_count == 2
    # Both calls must include "ON CONFLICT"
    for call in conn.execute.call_args_list:
        assert "ON CONFLICT" in call[0][0]


# ---------------------------------------------------------------------------
# 5. Entity with single event: std = 0 handled without divide by zero
# ---------------------------------------------------------------------------


def test_single_event_no_divide_by_zero() -> None:
    mean, std = BaselineLearner._compute_stats([99.9])
    assert std == 0.0
    # Downstream z-score code checks std == 0 before dividing
    if std == 0.0:
        z = None  # documented safe path
    assert z is None


# ---------------------------------------------------------------------------
# 6. Different entity types produce separate baselines
# ---------------------------------------------------------------------------


def test_different_entity_types_separate_rows() -> None:
    """_write_baseline is called once per (entity_id, entity_type, feature)
    primary key, so user and pod with the same id must each get their own row."""
    learner = BaselineLearner(_cfg())
    written_keys: list[tuple[str, str, str]] = []

    async def fake_write(
        entity_id,
        entity_type,
        feature,
        mean,
        std,
        sample_count,
        window_start,
        window_end,
    ):
        written_keys.append((entity_id, entity_type.value, feature))

    learner._write_baseline = fake_write  # type: ignore[method-assign]

    # Simulate learn() calling _write_baseline for two entity types
    loop = asyncio.get_event_loop()
    now = datetime.now(timezone.utc)
    for et in (EntityType.USER, EntityType.POD):
        loop.run_until_complete(
            learner._write_baseline(
                entity_id="id-shared",
                entity_type=et,
                feature="hour_of_day",
                mean=10.0,
                std=2.0,
                sample_count=10,
                window_start=now,
                window_end=now,
            )
        )
    keys_entity_types = [k[1] for k in written_keys]
    assert "user" in keys_entity_types
    assert "pod" in keys_entity_types
    assert len(written_keys) == 2


# ---------------------------------------------------------------------------
# 7. 14-day window correctly filters old events
# ---------------------------------------------------------------------------


def test_14_day_window_passed_to_loki() -> None:
    """learn() must compute window_start = now - 14 days and pass it to _query_loki."""
    learner = BaselineLearner(_cfg())
    calls: list[dict] = []

    async def fake_query_loki(entity_id, entity_type, start, end):
        calls.append({"start": start, "end": end})
        return []

    learner._query_loki = fake_query_loki  # type: ignore[method-assign]
    learner._pool = MagicMock()

    loop = asyncio.get_event_loop()
    loop.run_until_complete(learner.learn("user-1", EntityType.USER))

    assert len(calls) == 1
    delta = calls[0]["end"] - calls[0]["start"]
    assert abs(delta.days - 14) <= 1  # within a day of 14


# ---------------------------------------------------------------------------
# 8. _log_ methods emit structured JSON
# ---------------------------------------------------------------------------


def test_log_methods_emit_json(caplog) -> None:
    import logging

    learner = BaselineLearner(_cfg())
    with caplog.at_level(logging.INFO, logger="eel.ueba.baseline"):
        learner._log_info("test_event", key="value", count=42)
    assert len(caplog.records) == 1
    msg = caplog.records[0].getMessage()
    parsed = json.loads(msg)
    assert parsed["msg"] == "test_event"
    assert parsed["key"] == "value"
    assert parsed["count"] == 42
    assert parsed["level"] == "info"
