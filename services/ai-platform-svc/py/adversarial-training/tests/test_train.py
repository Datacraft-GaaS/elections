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

import numpy as np

from eel_adversarial_training.train import (
    AdversarialConfig,
    AdversarialTrainer,
    fgsm_attack,
    pgd_attack,
)


class _ConstantModel:
    """Predicts a fixed value; update returns a stub loss."""

    def __init__(self, value: float = 0.6) -> None:
        self._value = value

    def predict(self, x: np.ndarray) -> np.ndarray:
        return np.full(x.shape[0] if x.ndim > 1 else 1, self._value)

    def update(self, x: np.ndarray, y: np.ndarray) -> float:
        return 0.5


class _LinearModel:
    """Simple dot-product sigmoid model with gradient-compatible loss surface."""

    def __init__(self, weights: np.ndarray) -> None:
        self._w = weights.copy()

    def predict(self, x: np.ndarray) -> np.ndarray:
        flat = x.ravel()
        w = self._w[: flat.size]
        logit = float(np.dot(flat, w))
        return np.array([1.0 / (1.0 + np.exp(-logit))])

    def update(self, x: np.ndarray, y: np.ndarray) -> float:
        pred = self.predict(x)
        return float(-np.mean(y.ravel() * np.log(np.clip(pred, 1e-7, 1.0))))


def _make_xy(n: int = 4, d: int = 4, seed: int = 0) -> tuple[np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    x = rng.uniform(-1.0, 1.0, (n, d))
    y = rng.integers(0, 2, n).astype(float)
    return x, y


def test_fgsm_perturbs_input() -> None:
    x, y = _make_xy()
    rng = np.random.default_rng(1)
    model = _LinearModel(rng.uniform(-1, 1, x.shape[1]))
    x_adv = fgsm_attack(model, x[0:1], y[0:1], epsilon=0.03)
    assert not np.allclose(x_adv, x[0:1]), "FGSM should change the input"


def test_fgsm_l_inf_bounded() -> None:
    x, y = _make_xy()
    rng = np.random.default_rng(2)
    model = _LinearModel(rng.uniform(-1, 1, x.shape[1]))
    epsilon = 0.05
    x_adv = fgsm_attack(model, x[0:1], y[0:1], epsilon=epsilon)
    diff = np.abs(x_adv - x[0:1])
    assert diff.max() <= epsilon + 1e-9, f"L-inf bound violated: {diff.max()}"


def test_pgd_l_inf_bounded() -> None:
    x, y = _make_xy()
    rng = np.random.default_rng(3)
    model = _LinearModel(rng.uniform(-1, 1, x.shape[1]))
    epsilon = 0.03
    x_adv = pgd_attack(model, x[0:1], y[0:1], epsilon=epsilon, alpha=0.007, steps=5)
    diff = np.abs(x_adv - x[0:1])
    assert diff.max() <= epsilon + 1e-6, f"PGD L-inf bound violated: {diff.max()}"


def test_pgd_steps_parameter() -> None:
    x, y = _make_xy()
    rng = np.random.default_rng(4)
    weights = rng.uniform(-1, 1, x.shape[1])
    model = _LinearModel(weights)
    epsilon = 0.1
    alpha = 0.01

    # `steps` is the parameter under test, so vary it. The previous version ran
    # a single 40-step attack, dropped the FGSM result it computed, and asserted
    # nothing about `steps` at all -- it would have passed for any value.
    x_one = pgd_attack(model, x[0:1], y[0:1], epsilon=epsilon, alpha=alpha, steps=1)
    x_many = pgd_attack(model, x[0:1], y[0:1], epsilon=epsilon, alpha=alpha, steps=40)

    # Both must stay in the L-inf ball, at every step count.
    for name, x_adv in (("steps=1", x_one), ("steps=40", x_many)):
        assert np.max(np.abs(x_adv - x[0:1])) <= epsilon + 1e-6, f"{name} L-inf violated"
    # More steps must move further: each step adds alpha * sign(grad) before the
    # projection, so a longer run reaches the ball's edge rather than stopping
    # partway. Equal results would mean `steps` is ignored.
    assert np.max(np.abs(x_many - x[0:1])) > np.max(np.abs(x_one - x[0:1])), (
        "PGD with 40 steps must travel further than with 1"
    )
    # And the FGSM bound holds for its own single step.
    x_fgsm = fgsm_attack(model, x[0:1], y[0:1], epsilon=epsilon)
    assert np.max(np.abs(x_fgsm - x[0:1])) <= epsilon + 1e-6, "FGSM L-inf must be bounded"


def test_adversarial_trainer_returns_metrics() -> None:
    cfg = AdversarialConfig(pgd_steps=2, rng_seed=0)
    trainer = AdversarialTrainer(cfg)
    x, y = _make_xy(n=2, d=4)
    rng = np.random.default_rng(5)
    model = _LinearModel(rng.uniform(-1, 1, x.shape[1]))
    metrics = trainer.train_epoch(model, x[0:1], y[0:1])
    assert "loss" in metrics, "Metrics must contain 'loss'"
    assert "adv_loss" in metrics, "Metrics must contain 'adv_loss'"
