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
eel-ueba baseline-learner: queries Loki for Falco/Tetragon events,
computes per-entity per-feature mean+std over a 14-day rolling window,
and persists results to eel_ueba_baselines in PostgreSQL.
"""

from __future__ import annotations

import asyncio
import json
import logging
import math
import os
import statistics
import time
import yaml
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from enum import Enum
from typing import Any

import asyncpg
import httpx


# ---------------------------------------------------------------------------
# Domain model
# ---------------------------------------------------------------------------


class EntityType(str, Enum):
    USER = "user"
    SERVICE_ACCOUNT = "service_account"
    KIT = "kit"
    POD = "pod"


@dataclass
class FeatureVector:
    entity_id: str
    entity_type: EntityType
    hour_of_day: float
    query_rate_per_minute: float
    syscall_entropy: float
    bytes_egressed: float
    distinct_tables_accessed: float
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def as_dict(self) -> dict[str, float]:
        return {
            "hour_of_day": self.hour_of_day,
            "query_rate_per_minute": self.query_rate_per_minute,
            "syscall_entropy": self.syscall_entropy,
            "bytes_egressed": self.bytes_egressed,
            "distinct_tables_accessed": self.distinct_tables_accessed,
        }


# ---------------------------------------------------------------------------
# Learner
# ---------------------------------------------------------------------------


class BaselineLearner:
    def __init__(self, config: dict[str, Any]) -> None:
        self._cfg = config
        self._loki_url: str = config["loki"]["url"]
        self._interval: int = int(config["loki"]["query_interval_seconds"])
        self._window_days: int = int(config["baseline_window_days"])
        self._pg_dsn: str = config["postgres"]["dsn"]
        self._pool: asyncpg.Pool | None = None
        self._http: httpx.AsyncClient | None = None
        self._logger = logging.getLogger("eel.ueba.baseline")
        self._seen_entities: set[tuple[str, EntityType]] = set()

    # ------------------------------------------------------------------
    # Logging helpers
    # ------------------------------------------------------------------

    def _log_info(self, msg: str, **kwargs: Any) -> None:
        self._logger.info(json.dumps({"level": "info", "msg": msg, **kwargs}))

    def _log_error(self, msg: str, **kwargs: Any) -> None:
        self._logger.error(json.dumps({"level": "error", "msg": msg, **kwargs}))

    def _log_debug(self, msg: str, **kwargs: Any) -> None:
        self._logger.debug(json.dumps({"level": "debug", "msg": msg, **kwargs}))

    # ------------------------------------------------------------------
    # Loki query
    # ------------------------------------------------------------------

    async def _query_loki(
        self,
        entity_id: str,
        entity_type: EntityType,
        start: datetime,
        end: datetime,
    ) -> list[FeatureVector]:
        assert self._http is not None, "HTTP client not initialised"
        label = entity_type.value
        query = f'{{job=~"falco|tetragon",{label}_id="{entity_id}"}}'
        params = {
            "query": query,
            "start": str(int(start.timestamp() * 1e9)),
            "end": str(int(end.timestamp() * 1e9)),
            "limit": "5000",
        }
        try:
            resp = await self._http.get(
                f"{self._loki_url}/loki/api/v1/query_range",
                params=params,
                timeout=30.0,
            )
            resp.raise_for_status()
        except httpx.HTTPError as exc:
            self._log_error("loki_query_failed", entity_id=entity_id, error=str(exc))
            return []

        data = resp.json()
        vectors: list[FeatureVector] = []
        for stream in data.get("data", {}).get("result", []):
            for ts_ns, raw in stream.get("values", []):
                try:
                    event = json.loads(raw)
                except json.JSONDecodeError:
                    continue
                ts = datetime.fromtimestamp(int(ts_ns) / 1e9, tz=timezone.utc)
                vectors.append(
                    FeatureVector(
                        entity_id=entity_id,
                        entity_type=entity_type,
                        hour_of_day=float(ts.hour),
                        query_rate_per_minute=float(
                            event.get("query_rate_per_minute", 0.0)
                        ),
                        syscall_entropy=float(event.get("syscall_entropy", 0.0)),
                        bytes_egressed=float(event.get("bytes_egressed", 0.0)),
                        distinct_tables_accessed=float(
                            event.get("distinct_tables_accessed", 0.0)
                        ),
                        timestamp=ts,
                    )
                )
        return vectors

    # ------------------------------------------------------------------
    # Baseline computation
    # ------------------------------------------------------------------

    @staticmethod
    def _compute_stats(values: list[float]) -> tuple[float, float]:
        """Return (mean, std). std is 0.0 for single-sample or constant series."""
        if not values:
            return 0.0, 0.0
        mean = statistics.mean(values)
        if len(values) < 2:
            return mean, 0.0
        try:
            std = statistics.stdev(values)
        except statistics.StatisticsError:
            std = 0.0
        return mean, std

    # ------------------------------------------------------------------
    # Postgres writes
    # ------------------------------------------------------------------

    async def _ensure_schema(self) -> None:
        assert self._pool is not None
        async with self._pool.acquire() as conn:
            await conn.execute(
                """
				CREATE TABLE IF NOT EXISTS eel_ueba_baselines (
					entity_id    TEXT NOT NULL,
					entity_type  TEXT NOT NULL,
					feature      TEXT NOT NULL,
					mean         DOUBLE PRECISION NOT NULL,
					std          DOUBLE PRECISION NOT NULL,
					sample_count INTEGER NOT NULL,
					window_start TIMESTAMPTZ NOT NULL,
					window_end   TIMESTAMPTZ NOT NULL,
					updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
					PRIMARY KEY (entity_id, entity_type, feature)
				)
				"""
            )

    async def _write_baseline(
        self,
        entity_id: str,
        entity_type: EntityType,
        feature: str,
        mean: float,
        std: float,
        sample_count: int,
        window_start: datetime,
        window_end: datetime,
    ) -> None:
        assert self._pool is not None
        async with self._pool.acquire() as conn:
            await conn.execute(
                """
				INSERT INTO eel_ueba_baselines
					(entity_id, entity_type, feature, mean, std, sample_count, window_start, window_end, updated_at)
				VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
				ON CONFLICT (entity_id, entity_type, feature) DO UPDATE
					SET mean         = EXCLUDED.mean,
					    std          = EXCLUDED.std,
					    sample_count = EXCLUDED.sample_count,
					    window_start = EXCLUDED.window_start,
					    window_end   = EXCLUDED.window_end,
					    updated_at   = now()
				""",
                entity_id,
                entity_type.value,
                feature,
                mean,
                std,
                sample_count,
                window_start,
                window_end,
            )

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def learn(self, entity_id: str, entity_type: EntityType) -> None:
        """Query last 14 days, compute per-feature stats, persist baseline."""
        assert entity_id, "entity_id must be non-empty"
        assert isinstance(entity_type, EntityType), "entity_type must be EntityType"

        now = datetime.now(timezone.utc)
        window_start = now - timedelta(days=self._window_days)

        vectors = await self._query_loki(entity_id, entity_type, window_start, now)
        if not vectors:
            self._log_info(
                "no_events_for_entity",
                entity_id=entity_id,
                entity_type=entity_type.value,
            )
            return

        features: dict[str, list[float]] = {
            "hour_of_day": [],
            "query_rate_per_minute": [],
            "syscall_entropy": [],
            "bytes_egressed": [],
            "distinct_tables_accessed": [],
        }
        for vec in vectors:
            for feat, val in vec.as_dict().items():
                features[feat].append(val)

        for feature, values in features.items():
            mean, std = self._compute_stats(values)
            await self._write_baseline(
                entity_id=entity_id,
                entity_type=entity_type,
                feature=feature,
                mean=mean,
                std=std,
                sample_count=len(values),
                window_start=window_start,
                window_end=now,
            )

        self._log_info(
            "baseline_updated",
            entity_id=entity_id,
            entity_type=entity_type.value,
            sample_count=len(vectors),
        )

    async def _discover_entities(self, since: datetime) -> set[tuple[str, EntityType]]:
        """Discover entity IDs seen since a given time via Loki."""
        assert self._http is not None
        entities: set[tuple[str, EntityType]] = set()
        query = '{job=~"falco|tetragon"}'
        params = {
            "query": query,
            "start": str(int(since.timestamp() * 1e9)),
            "end": str(int(datetime.now(timezone.utc).timestamp() * 1e9)),
            "limit": "1000",
        }
        try:
            resp = await self._http.get(
                f"{self._loki_url}/loki/api/v1/query_range",
                params=params,
                timeout=30.0,
            )
            resp.raise_for_status()
        except httpx.HTTPError as exc:
            self._log_error("entity_discovery_failed", error=str(exc))
            return entities

        data = resp.json()
        for stream in data.get("data", {}).get("result", []):
            labels = stream.get("stream", {})
            for et in EntityType:
                eid = labels.get(f"{et.value}_id")
                if eid:
                    entities.add((eid, et))
        return entities

    async def run(self) -> None:
        """Infinite loop: discover entities, refresh baselines, sleep."""
        self._pool = await asyncpg.create_pool(self._pg_dsn, min_size=1, max_size=5)
        self._http = httpx.AsyncClient()
        try:
            await self._ensure_schema()
            self._log_info("baseline_learner_started", interval=self._interval)
            while True:
                interval_start = datetime.now(timezone.utc) - timedelta(
                    seconds=self._interval
                )
                try:
                    entities = await self._discover_entities(interval_start)
                    self._seen_entities.update(entities)
                except Exception as exc:
                    self._log_error("discovery_error", error=str(exc))

                for entity_id, entity_type in self._seen_entities:
                    try:
                        await self.learn(entity_id, entity_type)
                    except Exception as exc:
                        self._log_error(
                            "learn_error",
                            entity_id=entity_id,
                            entity_type=entity_type.value,
                            error=str(exc),
                        )
                await asyncio.sleep(self._interval)
        finally:
            await self._http.aclose()
            await self._pool.close()


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    config_path = os.environ.get("EEL_UEBA_CONFIG")
    assert config_path, "EEL_UEBA_CONFIG env var must point to the config YAML"
    with open(config_path) as fh:
        raw = yaml.safe_load(fh)
    cfg = raw["eel_ueba"]
    learner = BaselineLearner(cfg)
    asyncio.run(learner.run())
