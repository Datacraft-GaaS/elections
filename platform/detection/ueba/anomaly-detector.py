"""
eel-ueba anomaly-detector: tails Loki, computes z-scores against stored
baselines, emits alerts via PostgreSQL NOTIFY to the ECC dashboard.
"""

from __future__ import annotations

import asyncio
import json
import logging
import os
import time
import yaml
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

import asyncpg
import httpx

from baseline_learner import EntityType, FeatureVector


# ---------------------------------------------------------------------------
# Alert model
# ---------------------------------------------------------------------------


@dataclass
class Alert:
    entity_id: str
    entity_type: EntityType
    features_breached: list[str]
    z_scores: dict[str, float]
    severity: str  # "WARNING" | "CRITICAL"
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def to_json(self) -> str:
        return json.dumps(
            {
                "entity_id": self.entity_id,
                "entity_type": self.entity_type.value,
                "features_breached": self.features_breached,
                "z_scores": self.z_scores,
                "severity": self.severity,
                "timestamp": self.timestamp.isoformat(),
            }
        )


# ---------------------------------------------------------------------------
# Detector
# ---------------------------------------------------------------------------


class AnomalyDetector:
    def __init__(self, config: dict[str, Any]) -> None:
        self._cfg = config
        self._loki_url: str = config["loki"]["url"]
        self._pg_dsn: str = config["postgres"]["dsn"]
        self._channel: str = config["ecc_pubsub"]["channel"]
        self._threshold: float = float(config["zscore_threshold"])
        self._cooldown: int = int(config["alert_cooldown_seconds"])
        self._pool: asyncpg.Pool | None = None
        self._http: httpx.AsyncClient | None = None
        self._logger = logging.getLogger("eel.ueba.detector")
        # (entity_id, entity_type) -> last alert epoch seconds
        self._last_alert: dict[tuple[str, str], float] = {}

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
    # Baseline fetch
    # ------------------------------------------------------------------

    async def _fetch_baseline(
        self, entity_id: str, entity_type: EntityType
    ) -> dict[str, tuple[float, float]]:
        """Return {feature: (mean, std)} for this entity."""
        assert self._pool is not None
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(
                """
				SELECT feature, mean, std
				FROM eel_ueba_baselines
				WHERE entity_id = $1 AND entity_type = $2
				""",
                entity_id,
                entity_type.value,
            )
        return {row["feature"]: (row["mean"], row["std"]) for row in rows}

    # ------------------------------------------------------------------
    # Z-score and alerting
    # ------------------------------------------------------------------

    @staticmethod
    def _zscore(value: float, mean: float, std: float) -> float | None:
        """Return z-score, or None when std == 0 (no drift detectable)."""
        if std == 0.0:
            return None
        return (value - mean) / std

    async def _notify(self, alert: Alert) -> None:
        assert self._pool is not None
        async with self._pool.acquire() as conn:
            await conn.execute(
                f"SELECT pg_notify($1, $2)",
                self._channel,
                alert.to_json(),
            )

    async def detect(self, feature_vec: FeatureVector) -> list[Alert]:
        """Compute z-scores; return list of alerts (0 or 1 items)."""
        assert self._pool is not None, "pool not initialised"

        baseline = await self._fetch_baseline(
            feature_vec.entity_id, feature_vec.entity_type
        )
        if not baseline:
            self._log_debug(
                "no_baseline",
                entity_id=feature_vec.entity_id,
                entity_type=feature_vec.entity_type.value,
            )
            return []

        features = feature_vec.as_dict()
        breached: list[str] = []
        z_scores: dict[str, float] = {}
        max_z = 0.0

        for feat, val in features.items():
            if feat not in baseline:
                continue
            mean, std = baseline[feat]
            z = self._zscore(val, mean, std)
            if z is None:
                continue
            z_scores[feat] = z
            if abs(z) > self._threshold:
                breached.append(feat)
                if abs(z) > max_z:
                    max_z = abs(z)

        if not breached:
            return []

        # Cooldown check
        key = (feature_vec.entity_id, feature_vec.entity_type.value)
        now_ts = time.time()
        if now_ts - self._last_alert.get(key, 0.0) < self._cooldown:
            self._log_debug(
                "alert_suppressed_by_cooldown",
                entity_id=feature_vec.entity_id,
            )
            return []

        severity = "CRITICAL" if max_z > 5.0 else "WARNING"
        alert = Alert(
            entity_id=feature_vec.entity_id,
            entity_type=feature_vec.entity_type,
            features_breached=breached,
            z_scores={f: z_scores[f] for f in breached},
            severity=severity,
        )
        self._last_alert[key] = now_ts
        await self._notify(alert)
        self._log_info(
            "alert_raised",
            entity_id=feature_vec.entity_id,
            entity_type=feature_vec.entity_type.value,
            severity=severity,
            features=breached,
        )
        return [alert]

    # ------------------------------------------------------------------
    # Loki tail stream
    # ------------------------------------------------------------------

    async def _parse_loki_event(self, raw: str) -> FeatureVector | None:
        try:
            event = json.loads(raw)
        except json.JSONDecodeError:
            return None
        entity_id = event.get("entity_id")
        entity_type_str = event.get("entity_type")
        if not entity_id or not entity_type_str:
            return None
        try:
            entity_type = EntityType(entity_type_str)
        except ValueError:
            return None
        ts_str = event.get("timestamp")
        try:
            ts = (
                datetime.fromisoformat(ts_str) if ts_str else datetime.now(timezone.utc)
            )
        except (ValueError, TypeError):
            ts = datetime.now(timezone.utc)
        return FeatureVector(
            entity_id=entity_id,
            entity_type=entity_type,
            hour_of_day=float(event.get("hour_of_day", ts.hour)),
            query_rate_per_minute=float(event.get("query_rate_per_minute", 0.0)),
            syscall_entropy=float(event.get("syscall_entropy", 0.0)),
            bytes_egressed=float(event.get("bytes_egressed", 0.0)),
            distinct_tables_accessed=float(event.get("distinct_tables_accessed", 0.0)),
            timestamp=ts,
        )

    async def run(self) -> None:
        """Tail Loki stream and detect anomalies in real time."""
        self._pool = await asyncpg.create_pool(self._pg_dsn, min_size=1, max_size=5)
        self._http = httpx.AsyncClient(timeout=None)
        self._log_info("anomaly_detector_started")
        try:
            query = '{job=~"falco|tetragon"}'
            params = {
                "query": query,
                "delay_for": "0",
                "limit": "100",
                "start": str(int(datetime.now(timezone.utc).timestamp() * 1e9)),
            }
            async with self._http.stream(
                "GET",
                f"{self._loki_url}/loki/api/v1/tail",
                params=params,
            ) as stream:
                async for line in stream.aiter_lines():
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        payload = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    for stream_block in payload.get("streams", []):
                        for _ts, raw in stream_block.get("values", []):
                            vec = await self._parse_loki_event(raw)
                            if vec:
                                try:
                                    await self.detect(vec)
                                except Exception as exc:
                                    self._log_error("detect_error", error=str(exc))
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
    detector = AnomalyDetector(cfg)
    asyncio.run(detector.run())
