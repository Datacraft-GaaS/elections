from __future__ import annotations

# Adversarial training loop with PGD, FGSM, AutoAttack adversarial examples.
# Implemented using numpy only (no torch dependency for basic CI).
#
# References:
#   FGSM: Goodfellow, Shlens, Szegedy "Explaining and Harnessing Adversarial
#         Examples" ICLR 2015.
#   PGD:  Madry et al. "Towards Deep Learning Models Resistant to Adversarial
#         Attacks" ICLR 2018.
#   AutoAttack: Croce, Hein "Reliable evaluation of adversarial robustness with
#         an ensemble of diverse parameter-free attacks" ICML 2020.

from typing import Protocol, runtime_checkable

import numpy as np
from pydantic import BaseModel, ConfigDict


class AdversarialConfig(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    epsilon: float = 0.03
    alpha: float = 0.007
    pgd_steps: int = 40
    ensemble_k: int = 3
    ensemble_n: int = 5
    rng_seed: int = 42


@runtime_checkable
class ModelProtocol(Protocol):
    def predict(self, x: np.ndarray) -> np.ndarray: ...
    def update(self, x: np.ndarray, y: np.ndarray) -> float: ...


def fgsm_attack(
    model: ModelProtocol,
    x: np.ndarray,
    y: np.ndarray,
    epsilon: float,
) -> np.ndarray:
    """Single-step FGSM perturbation (sign-gradient approximation via finite differences)."""
    grad = _finite_diff_grad(model, x, y)
    return x + epsilon * np.sign(grad)


def pgd_attack(
    model: ModelProtocol,
    x: np.ndarray,
    y: np.ndarray,
    epsilon: float,
    alpha: float,
    steps: int,
) -> np.ndarray:
    """PGD attack with L-infinity ball constraint: clip to [x - epsilon, x + epsilon]."""
    x_adv = x.copy()
    for _ in range(steps):
        grad = _finite_diff_grad(model, x_adv, y)
        x_adv = x_adv + alpha * np.sign(grad)
        # Project back into the L-inf ball around x.
        x_adv = np.clip(x_adv, x - epsilon, x + epsilon)
    return x_adv


def auto_attack_step(
    model: ModelProtocol,
    x: np.ndarray,
    y: np.ndarray,
    epsilon: float,
) -> np.ndarray:
    """APGD step (PGD with momentum) as a simplified AutoAttack surrogate."""
    alpha = epsilon / 4.0
    steps = 10
    momentum = 0.75
    x_adv = x.copy()
    velocity = np.zeros_like(x)
    for _ in range(steps):
        grad = _finite_diff_grad(model, x_adv, y)
        velocity = momentum * velocity + (1.0 - momentum) * np.sign(grad)
        x_adv = x_adv + alpha * np.sign(velocity)
        x_adv = np.clip(x_adv, x - epsilon, x + epsilon)
    return x_adv


def _finite_diff_grad(
    model: ModelProtocol,
    x: np.ndarray,
    y: np.ndarray,
    h: float = 1e-3,
) -> np.ndarray:
    """Numerical gradient of cross-entropy loss via finite differences."""
    base_loss = _cross_entropy(model.predict(x), y)
    grad = np.zeros_like(x, dtype=np.float64)
    flat = x.ravel()
    for i in range(flat.size):
        x_plus = x.copy()
        x_plus.ravel()[i] += h
        loss_plus = _cross_entropy(model.predict(x_plus), y)
        grad.ravel()[i] = (loss_plus - base_loss) / h
    return grad


def _cross_entropy(pred: np.ndarray, y: np.ndarray) -> float:
    """Binary cross-entropy. pred and y are 1-D arrays of length N."""
    p = np.clip(pred.ravel(), 1e-7, 1.0 - 1e-7)
    t = y.ravel().astype(np.float64)
    return float(-np.mean(t * np.log(p) + (1.0 - t) * np.log(1.0 - p)))


def _accuracy(pred: np.ndarray, y: np.ndarray) -> float:
    labels = (pred.ravel() >= 0.5).astype(int)
    return float(np.mean(labels == y.ravel().astype(int)))


class AdversarialTrainer:
    def __init__(self, config: AdversarialConfig) -> None:
        self._config = config

    def train_epoch(
        self,
        model: ModelProtocol,
        x: np.ndarray,
        y: np.ndarray,
    ) -> dict[str, float]:
        """Train one epoch: clean forward pass + adversarial update."""
        clean_loss = _cross_entropy(model.predict(x), y)
        x_adv = pgd_attack(
            model,
            x,
            y,
            epsilon=self._config.epsilon,
            alpha=self._config.alpha,
            steps=self._config.pgd_steps,
        )
        adv_loss = model.update(x_adv, y)
        return {"loss": clean_loss, "adv_loss": adv_loss}

    def evaluate_robustness(
        self,
        model: ModelProtocol,
        x: np.ndarray,
        y: np.ndarray,
    ) -> dict[str, float]:
        """Report clean_acc, fgsm_acc, pgd_acc."""
        clean_acc = _accuracy(model.predict(x), y)

        x_fgsm = fgsm_attack(model, x, y, epsilon=self._config.epsilon)
        fgsm_acc = _accuracy(model.predict(x_fgsm), y)

        x_pgd = pgd_attack(
            model,
            x,
            y,
            epsilon=self._config.epsilon,
            alpha=self._config.alpha,
            steps=self._config.pgd_steps,
        )
        pgd_acc = _accuracy(model.predict(x_pgd), y)

        return {"clean_acc": clean_acc, "fgsm_acc": fgsm_acc, "pgd_acc": pgd_acc}
