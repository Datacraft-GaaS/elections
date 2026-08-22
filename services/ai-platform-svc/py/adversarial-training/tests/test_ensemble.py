from __future__ import annotations

import numpy as np

from eel_adversarial_training.ensemble import EnsembleConfig, EnsembleVoter
from eel_adversarial_training.train import ModelProtocol


class _FixedModel:
    """Always predicts the given value."""

    def __init__(self, value: float) -> None:
        self._v = value

    def predict(self, x: np.ndarray) -> np.ndarray:
        return np.array([self._v])

    def update(self, x: np.ndarray, y: np.ndarray) -> float:
        return 0.0


def _dummy_x() -> np.ndarray:
    return np.zeros((1, 4))


def test_ensemble_unanimous_above_threshold() -> None:
    # k=3, n=3 -- all three agree -> above threshold.
    cfg = EnsembleConfig(k=3, n=3)
    models: list[ModelProtocol] = [_FixedModel(0.9), _FixedModel(0.8), _FixedModel(0.7)]
    voter = EnsembleVoter(cfg, models)
    pred, conf, above = voter.vote(_dummy_x())
    assert pred == 1
    assert above is True


def test_ensemble_majority_fails_k_of_n() -> None:
    # k=3, n=5 -- only 2 agree on label=1 -> not above threshold.
    cfg = EnsembleConfig(k=3, n=5)
    models: list[ModelProtocol] = [
        _FixedModel(0.9),
        _FixedModel(0.8),
        _FixedModel(0.1),
        _FixedModel(0.2),
        _FixedModel(0.3),
    ]
    voter = EnsembleVoter(cfg, models)
    _, _, above = voter.vote(_dummy_x())
    # 3 models predict 0, 2 predict 1; majority is 0 with 3 agrees = k=3 exactly.
    # Adjust: use k=4 to force failure.
    cfg2 = EnsembleConfig(k=4, n=5)
    voter2 = EnsembleVoter(cfg2, models)
    _, _, above2 = voter2.vote(_dummy_x())
    assert above2 is False


def test_ensemble_confidence_range() -> None:
    cfg = EnsembleConfig(k=2, n=4)
    models: list[ModelProtocol] = [
        _FixedModel(0.9),
        _FixedModel(0.8),
        _FixedModel(0.2),
        _FixedModel(0.1),
    ]
    voter = EnsembleVoter(cfg, models)
    _, conf, _ = voter.vote(_dummy_x())
    assert 0.0 <= conf <= 1.0, f"Confidence out of range: {conf}"


def test_ensemble_vote_returns_tuple() -> None:
    cfg = EnsembleConfig(k=1, n=2)
    models: list[ModelProtocol] = [_FixedModel(0.6), _FixedModel(0.4)]
    voter = EnsembleVoter(cfg, models)
    result = voter.vote(_dummy_x())
    assert isinstance(result, tuple)
    assert len(result) == 3
    pred, conf, above = result
    assert isinstance(pred, int)
    assert isinstance(conf, float)
    assert isinstance(above, bool)
