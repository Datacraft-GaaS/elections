from __future__ import annotations

import math
from collections import defaultdict
from datetime import datetime, timezone

import numpy as np
from sklearn.ensemble import GradientBoostingRegressor
from uuid6 import uuid7

from turnout_forecast.views import TurnoutForecast, TurnoutIngest, VarianceAlert

_VARIANCE_THRESHOLD = 10.0  # percentage points

# station_code -> list of ingested points
_station_data: dict[str, list[TurnoutIngest]] = defaultdict(list)
_alerts: list[VarianceAlert] = []


def _uuid7str() -> str:
    return str(uuid7())


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _sigmoid_pct(hour: int) -> float:
    """Baseline S-curve: turnout fraction = sigmoid(hour/24 * 6 - 3)."""
    x = (hour / 24.0) * 6.0 - 3.0
    return 100.0 * (1.0 / (1.0 + math.exp(-x)))


async def ingest(point: TurnoutIngest) -> None:
    _station_data[point.station_code].append(point)


async def forecast(
    station_code: str,
    current_hour: int,
    registered_voters: int,
    votes_cast: int,
) -> TurnoutForecast:
    points = _station_data.get(station_code, [])

    if len(points) >= 3:
        X = np.array([[p.hour, p.registered_voters] for p in points], dtype=float)
        y = np.array(
            [p.votes_cast_so_far / p.registered_voters * 100.0 for p in points], dtype=float
        )
        gb = GradientBoostingRegressor(n_estimators=50, random_state=42)
        gb.fit(X, y)
        pred = float(gb.predict([[current_hour, registered_voters]])[0])
    else:
        # Sigmoid baseline with current observation blend
        baseline = _sigmoid_pct(current_hour)
        observed = votes_cast / registered_voters * 100.0 if registered_voters > 0 else baseline
        pred = 0.5 * baseline + 0.5 * observed

    pred = max(0.0, min(100.0, pred))
    margin = 5.0

    return TurnoutForecast(
        forecast_id=_uuid7str(),
        station_code=station_code,
        forecasted_turnout_pct=round(pred, 2),
        lower_bound_pct=max(0.0, round(pred - margin, 2)),
        upper_bound_pct=min(100.0, round(pred + margin, 2)),
        variance_alert=False,
        generated_at=_now(),
    )


async def check_variance(
    station_code: str,
    fc: TurnoutForecast,
    actual_pct: float,
) -> VarianceAlert | None:
    deviation = abs(actual_pct - fc.forecasted_turnout_pct)
    if deviation <= _VARIANCE_THRESHOLD:
        return None
    alert = VarianceAlert(
        alert_id=_uuid7str(),
        station_code=station_code,
        forecasted_pct=fc.forecasted_turnout_pct,
        actual_pct=actual_pct,
        deviation_pct=round(deviation, 2),
        alerted_at=_now(),
    )
    _alerts.append(alert)
    return alert


def get_alerts() -> list[VarianceAlert]:
    return list(_alerts)


def clear_state() -> None:
    _station_data.clear()
    _alerts.clear()
