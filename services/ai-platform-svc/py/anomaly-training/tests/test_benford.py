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

import math

import numpy as np
import pandas as pd
import pytest

from eel_anomaly_training.benford import (
    benford_chi2_test,
    expected_benford,
    fit_benford,
    fit_county_benford,
    fit_second_digit_benford,
)


def test_expected_benford_sums_to_one() -> None:
    dist = expected_benford()
    assert abs(sum(dist.values()) - 1.0) < 1e-9
    assert set(dist.keys()) == set(range(1, 10))


def test_first_digit_distribution_on_known() -> None:
    # Powers of 10 follow Benford exactly in the limit.
    # Use a large sample drawn from Benford's distribution.
    rng = np.random.default_rng(42)
    expected = expected_benford()
    digits = list(range(1, 10))
    probs = [expected[d] for d in digits]
    samples = rng.choice(digits, size=50_000, p=probs)
    df = pd.DataFrame({"x": samples})
    observed = fit_benford(df, "x")
    for d in range(1, 10):
        assert abs(observed[d] - expected[d]) < 0.02, (
            f"digit {d} off: {observed[d]:.4f} vs {expected[d]:.4f}"
        )


def test_chi2_on_benford_data() -> None:
    rng = np.random.default_rng(0)
    expected = expected_benford()
    digits = list(range(1, 10))
    probs = [expected[d] for d in digits]
    samples = rng.choice(digits, size=10_000, p=probs)
    df = pd.DataFrame({"x": samples})
    observed = fit_benford(df, "x")
    _, p_value = benford_chi2_test(observed, n_samples=10_000)
    # Benford-distributed data should not be rejected
    assert p_value > 0.01, f"p_value too low: {p_value}"


def test_second_digit_benford() -> None:
    # Generate multi-digit numbers
    rng = np.random.default_rng(7)
    vals = rng.integers(10, 999_999, size=5_000)
    df = pd.DataFrame({"x": vals})
    result = fit_second_digit_benford(df, "x")
    assert set(result.keys()) == set(range(0, 10))
    assert abs(sum(result.values()) - 1.0) < 1e-9


def test_county_benford_keys() -> None:
    from eel_anomaly_training.loader import KenyaHistoricalDataLoader

    loader = KenyaHistoricalDataLoader()
    df = loader.load_2013()
    result = fit_county_benford(df)
    assert len(result) == 47
    for county_id in range(1, 48):
        assert county_id in result
        county = result[county_id]
        assert "observed" in county
        assert "expected" in county
        assert "chi2_stat" in county
        assert "p_value" in county
