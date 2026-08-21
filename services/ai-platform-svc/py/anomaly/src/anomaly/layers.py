from __future__ import annotations

import time
from collections import defaultdict
from datetime import datetime, timezone
from typing import Literal

import numpy as np
from sklearn.ensemble import IsolationForest  # type: ignore[import-untyped]
from uuid6 import uuid7

from anomaly.views import AnomalyAlert, TelemetryPoint

_Severity = Literal["low", "medium", "high", "critical"]


def _uuid7str() -> str:
    return str(uuid7())


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _severity(score: float) -> _Severity:
    if score >= 0.9:
        return "critical"
    if score >= 0.7:
        return "high"
    if score >= 0.4:
        return "medium"
    return "low"


# Layer 1: IsoForest -- trained on small synthetic baseline at import time
_iso_model: IsolationForest = IsolationForest(n_estimators=50, contamination=0.1, random_state=42)  # type: ignore[arg-type]
_synthetic_X = np.array([[100 + i, 100 + i, 2, 0.02, 1.0] for i in range(40)], dtype=float)
_iso_model.fit(_synthetic_X)


def _featurize(p: TelemetryPoint) -> list[float]:
    ratio_spoilt = p.spoilt_ballots / p.votes_cast if p.votes_cast > 0 else 0.0
    ratio_issued = p.ballot_papers_issued / p.votes_cast if p.votes_cast > 0 else 1.0
    return [
        float(p.votes_cast),
        float(p.ballot_papers_issued),
        float(p.spoilt_ballots),
        ratio_spoilt,
        ratio_issued,
    ]


async def layer1_isoforest(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    alerts: list[AnomalyAlert] = []
    if not points:
        return alerts
    X = np.array([_featurize(p) for p in points], dtype=float)
    scores = _iso_model.decision_function(X)
    preds = _iso_model.predict(X)
    for p, pred, raw in zip(points, preds, scores):
        if pred == -1:
            norm_score = float(np.clip(1.0 - (raw + 0.5), 0.0, 1.0))
            alerts.append(
                AnomalyAlert(
                    alert_id=_uuid7str(),
                    station_code=p.station_code,
                    layer="layer1-isoforest",
                    score=max(norm_score, 0.01),
                    severity=_severity(norm_score),
                    details={"raw_score": float(raw)},
                    detected_at=_now(),
                )
            )
    return alerts


async def layer2_cross_source(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    alerts: list[AnomalyAlert] = []
    for p in points:
        diff = abs(p.votes_cast - p.ballot_papers_issued)
        threshold = max(p.ballot_papers_issued * 0.2, 10)
        if diff > threshold:
            score = min(diff / max(p.ballot_papers_issued, 1), 1.0)
            alerts.append(
                AnomalyAlert(
                    alert_id=_uuid7str(),
                    station_code=p.station_code,
                    layer="layer2-cross-source",
                    score=max(score, 0.01),
                    severity=_severity(score),
                    details={
                        "votes_cast": p.votes_cast,
                        "ballot_papers_issued": p.ballot_papers_issued,
                        "diff": diff,
                    },
                    detected_at=_now(),
                )
            )
    return alerts


async def layer3_benford(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    from scipy import stats as scipy_stats  # type: ignore[import-untyped]

    alerts: list[AnomalyAlert] = []
    if len(points) < 10:
        return alerts

    expected = np.array([np.log10(1 + 1 / d) for d in range(1, 10)])
    counts = np.zeros(9)
    total = 0
    for p in points:
        for val in (p.votes_cast, p.ballot_papers_issued):
            if val > 0:
                first = int(str(val)[0])
                counts[first - 1] += 1
                total += 1

    if total < 10:
        return alerts

    observed = counts / total
    chi2, pval = scipy_stats.chisquare(observed + 1e-9, f_exp=expected)
    if pval < 0.05:
        score = min(chi2 / 100.0, 1.0)
        alerts.append(
            AnomalyAlert(
                alert_id=_uuid7str(),
                station_code="BATCH",
                layer="layer3-benford",
                score=max(score, 0.01),
                severity=_severity(score),
                details={"chi2": float(chi2), "p_value": float(pval)},
                detected_at=_now(),
            )
        )
    return alerts


async def layer4_cryptographic(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    import re

    alerts: list[AnomalyAlert] = []
    # Valid station codes: alphanumeric + underscore, 3-20 chars
    pattern = re.compile(r"^[A-Z0-9_]{3,20}$")
    for p in points:
        if not pattern.match(p.station_code):
            alerts.append(
                AnomalyAlert(
                    alert_id=_uuid7str(),
                    station_code=p.station_code,
                    layer="layer4-cryptographic",
                    score=0.8,
                    severity="high",
                    details={"reason": "invalid station_code format"},
                    detected_at=_now(),
                )
            )
    return alerts


async def layer5_canary(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    alerts: list[AnomalyAlert] = []
    for p in points:
        if p.station_code.upper().startswith("CANARY"):
            alerts.append(
                AnomalyAlert(
                    alert_id=_uuid7str(),
                    station_code=p.station_code,
                    layer="layer5-canary",
                    score=1.0,
                    severity="critical",
                    details={"reason": "canary station code detected in production data"},
                    detected_at=_now(),
                )
            )
    return alerts


# Convergence tracking state
_running_sums: dict[str, list[float]] = defaultdict(list)


async def layer6_convergence(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    alerts: list[AnomalyAlert] = []
    for p in points:
        key = p.station_code
        ratio = p.votes_cast / p.ballot_papers_issued if p.ballot_papers_issued > 0 else 0.0
        history = _running_sums[key]
        history.append(ratio)
        if len(history) >= 5:
            mean = sum(history) / len(history)
            deviation = abs(ratio - mean)
            if deviation > 0.3 and mean > 0:
                rel_dev = deviation / mean
                score = min(rel_dev, 1.0)
                alerts.append(
                    AnomalyAlert(
                        alert_id=_uuid7str(),
                        station_code=p.station_code,
                        layer="layer6-convergence",
                        score=max(score, 0.01),
                        severity=_severity(score),
                        details={
                            "current_ratio": ratio,
                            "mean_ratio": mean,
                            "deviation": deviation,
                        },
                        detected_at=_now(),
                    )
                )
    return alerts


# Rate-limit state: station_code -> last alert timestamp
_last_alert_ts: dict[str, float] = {}
_RATE_LIMIT_SECS = 60.0


async def layer7_dedup_ratelimit(alerts: list[AnomalyAlert]) -> list[AnomalyAlert]:
    now_ts = time.monotonic()
    filtered: list[AnomalyAlert] = []
    seen: set[str] = set()
    for alert in alerts:
        key = f"{alert.station_code}:{alert.layer}"
        last = _last_alert_ts.get(key, 0.0)
        if (now_ts - last) >= _RATE_LIMIT_SECS and key not in seen:
            filtered.append(alert)
            _last_alert_ts[key] = now_ts
            seen.add(key)
    return filtered


async def detect_all(points: list[TelemetryPoint]) -> list[AnomalyAlert]:
    raw: list[AnomalyAlert] = []
    for layer_fn in (
        layer1_isoforest,
        layer2_cross_source,
        layer3_benford,
        layer4_cryptographic,
        layer5_canary,
        layer6_convergence,
    ):
        raw.extend(await layer_fn(points))
    return await layer7_dedup_ratelimit(raw)
