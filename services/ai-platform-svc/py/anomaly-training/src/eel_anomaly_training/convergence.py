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
from scipy import stats


def ks_divergence_test(
    stream_a: np.ndarray, stream_b: np.ndarray
) -> tuple[float, float]:
    """Two-sample Kolmogorov-Smirnov test.

    Returns (statistic, p_value). Small p means distributions differ.
    """
    result = stats.ks_2samp(stream_a, stream_b)
    return float(result.statistic), float(result.pvalue)


def wasserstein_distance(stream_a: np.ndarray, stream_b: np.ndarray) -> float:
    """Earth mover's distance between two 1-D distributions."""
    return float(stats.wasserstein_distance(stream_a, stream_b))


def detect_paired_tally_divergence(
    kit_tallies: np.ndarray,
    paper_tallies: np.ndarray,
    alpha: float = 0.05,
) -> bool:
    """Return True if kit and paper tally distributions diverge significantly.

    Uses a two-sample KS test at the given significance level.
    """
    _, p_value = ks_divergence_test(kit_tallies, paper_tallies)
    return p_value < alpha
