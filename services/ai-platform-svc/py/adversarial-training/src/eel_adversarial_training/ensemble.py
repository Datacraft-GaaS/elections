from __future__ import annotations

# K-of-N ensemble voting harness.

import numpy as np
from pydantic import BaseModel, ConfigDict

from eel_adversarial_training.train import ModelProtocol


class EnsembleConfig(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    k: int
    n: int
    agreement_threshold: float = 0.6


class EnsembleVoter:
    def __init__(self, config: EnsembleConfig, models: list[ModelProtocol]) -> None:
        assert len(models) == config.n, f"Expected {config.n} models, got {len(models)}"
        self._config = config
        self._models = models

    def vote(self, x: np.ndarray) -> tuple[int, float, bool]:
        """Return (prediction, confidence, meets_threshold).

        meets_threshold is True iff at least k of n models predict the same
        majority class.
        """
        preds = np.array([int(m.predict(x).ravel()[0] >= 0.5) for m in self._models])
        ones = int(np.sum(preds == 1))
        zeros = int(np.sum(preds == 0))

        if ones >= zeros:
            majority = 1
            agree_count = ones
        else:
            majority = 0
            agree_count = zeros

        confidence = agree_count / self._config.n
        meets_threshold = agree_count >= self._config.k
        return (majority, confidence, meets_threshold)

    def is_above_threshold(self, x: np.ndarray) -> bool:
        _, _, meets = self.vote(x)
        return meets
