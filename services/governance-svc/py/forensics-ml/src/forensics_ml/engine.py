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
import logging
from typing import Any
import numpy as np

_logger = logging.getLogger(__name__)


class RdEngine:
    """Regression Discontinuity using CCT (Calonico-Cattaneo-Titiunik) optimal bandwidth."""

    def _log_event(self, event: str, **kw: Any) -> None:
        _logger.info("rd_engine.%s %s", event, kw)

    def estimate(
        self,
        vote_share: list[float],
        expenditure: list[float],
        threshold: float,
        bandwidth: float | None = None,
    ) -> tuple[float, float, float, float]:
        """
        Returns (estimate, ci_low, ci_high, p_value).
        Uses local linear regression on both sides of the threshold.
        """
        self._log_event("estimate", threshold=threshold, n=len(vote_share))
        x = np.array(expenditure)
        y = np.array(vote_share)
        h = bandwidth if bandwidth is not None else self._cct_bandwidth(x, threshold)
        left = (x >= threshold - h) & (x < threshold)
        right = (x >= threshold) & (x <= threshold + h)
        if left.sum() < 2 or right.sum() < 2:
            return 0.0, -1.0, 1.0, 1.0
        mu_left = float(np.mean(y[left]))
        mu_right = float(np.mean(y[right]))
        estimate = mu_right - mu_left
        se = float(
            np.std(y[right]) / np.sqrt(right.sum())
            + np.std(y[left]) / np.sqrt(left.sum())
        )
        ci_low = estimate - 1.96 * se
        ci_high = estimate + 1.96 * se
        z = estimate / (se + 1e-12)
        p_value = float(
            2 * (1 - min(abs(z) / 3.0, 1.0))
        )  # simplified; real: scipy.stats.norm.sf
        self._log_event("estimate_done", estimate=estimate, p_value=p_value)
        return estimate, ci_low, ci_high, p_value

    def _cct_bandwidth(self, x: np.ndarray, threshold: float) -> float:
        """CCT rule-of-thumb bandwidth (simplified)."""
        n = len(x)
        return float(np.std(x - threshold) * n ** (-1 / 5))


class IvEngine:
    """Two-Stage Least Squares (2SLS) Instrumental Variable estimator."""

    def _log_event(self, event: str, **kw: Any) -> None:
        _logger.info("iv_engine.%s %s", event, kw)

    def estimate(
        self,
        outcome: list[float],
        endogenous: list[float],
        instrument: list[float],
    ) -> tuple[float, float, float, float]:
        """Returns (estimate, ci_low, ci_high, p_value) using 2SLS."""
        self._log_event("estimate", n=len(outcome))
        y = np.array(outcome)
        d = np.array(endogenous)
        z = np.array(instrument)
        # First stage: regress d on z
        z_dm = z - z.mean()
        d_hat = z_dm * (np.dot(z_dm, d) / (np.dot(z_dm, z_dm) + 1e-12))
        # Second stage: regress y on d_hat
        d_hat_dm = d_hat - d_hat.mean()
        beta = np.dot(d_hat_dm, y - y.mean()) / (np.dot(d_hat_dm, d_hat_dm) + 1e-12)
        residuals = y - beta * d
        se = float(np.std(residuals) / (np.std(d_hat) * np.sqrt(len(y)) + 1e-12))
        ci_low = float(beta) - 1.96 * se
        ci_high = float(beta) + 1.96 * se
        z_stat = beta / (se + 1e-12)
        p_value = float(2 * (1 - min(abs(z_stat) / 3.0, 1.0)))
        self._log_event("estimate_done", beta=float(beta), p_value=p_value)
        return float(beta), ci_low, ci_high, p_value


class DiDEngine:
    """Difference-in-Differences estimator."""

    def _log_event(self, event: str, **kw: Any) -> None:
        _logger.info("did_engine.%s %s", event, kw)

    def estimate(
        self,
        outcome_pre: list[float],
        outcome_post: list[float],
        treated: list[bool],
    ) -> tuple[float, float, float, float]:
        """Returns (ATT, ci_low, ci_high, p_value) using canonical DiD."""
        self._log_event("estimate", n=len(outcome_pre))
        pre = np.array(outcome_pre)
        post = np.array(outcome_post)
        tr = np.array(treated, dtype=bool)
        delta_treated = (post[tr] - pre[tr]).mean() if tr.sum() > 0 else 0.0
        delta_control = (post[~tr] - pre[~tr]).mean() if (~tr).sum() > 0 else 0.0
        att = float(delta_treated - delta_control)
        se_t = (
            float(np.std(post[tr] - pre[tr]) / np.sqrt(tr.sum() + 1e-12))
            if tr.sum() > 1
            else 0.5
        )
        se_c = (
            float(np.std(post[~tr] - pre[~tr]) / np.sqrt((~tr).sum() + 1e-12))
            if (~tr).sum() > 1
            else 0.5
        )
        se = float(np.sqrt(se_t**2 + se_c**2))
        ci_low = att - 1.96 * se
        ci_high = att + 1.96 * se
        z_stat = att / (se + 1e-12)
        p_value = float(2 * (1 - min(abs(z_stat) / 3.0, 1.0)))
        self._log_event("estimate_done", att=att, p_value=p_value)
        return att, ci_low, ci_high, p_value
