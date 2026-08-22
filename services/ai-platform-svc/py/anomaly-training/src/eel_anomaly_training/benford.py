from __future__ import annotations

import math
from typing import Any

import numpy as np
import pandas as pd
from scipy import stats


def expected_benford() -> dict[int, float]:
    """Theoretical Benford first-digit distribution."""
    return {d: math.log10(1 + 1 / d) for d in range(1, 10)}


def fit_benford(df: pd.DataFrame, column: str) -> dict[int, float]:
    """Observed first-digit frequencies for a numeric column."""
    values = df[column].dropna().astype(str).str.lstrip("-0.")
    first_digits = values.str[:1]
    valid = first_digits[first_digits.str.isdigit() & (first_digits != "0")]
    counts = valid.astype(int).value_counts().sort_index()
    total = counts.sum()
    if total == 0:
        return {d: 0.0 for d in range(1, 10)}
    freq: dict[int, float] = {}
    for d in range(1, 10):
        freq[d] = float(counts.get(d, 0)) / float(total)
    return freq


def fit_second_digit_benford(df: pd.DataFrame, column: str) -> dict[int, float]:
    """Observed second-digit frequencies (0-9) for a numeric column."""
    values = df[column].dropna().abs().astype(str)
    # strip leading zeros and decimal point, take second digit
    second_digits: list[int] = []
    for v in values:
        digits = v.replace(".", "").lstrip("0")
        if len(digits) >= 2:
            second_digits.append(int(digits[1]))
    if not second_digits:
        return {d: 0.0 for d in range(0, 10)}
    arr = np.array(second_digits)
    total = len(arr)
    return {d: float((arr == d).sum()) / float(total) for d in range(0, 10)}


def benford_chi2_test(
    observed: dict[int, float], n_samples: int
) -> tuple[float, float]:
    """Chi-squared goodness-of-fit against Benford's law.

    Returns (chi2_stat, p_value).
    Only applies to first-digit distributions (keys 1-9).
    """
    expected = expected_benford()
    digits = sorted(set(observed) & set(expected))
    obs_counts = np.array([observed[d] * n_samples for d in digits])
    exp_counts = np.array([expected[d] * n_samples for d in digits])
    result = stats.chisquare(obs_counts, f_exp=exp_counts)
    return float(result.statistic), float(result.pvalue)


def fit_county_benford(df: pd.DataFrame) -> dict[int, dict[str, Any]]:
    """Per-county Benford analysis with confidence intervals on votes_cast."""
    result: dict[int, dict[str, Any]] = {}
    for county_id, group in df.groupby("county_id"):
        observed = fit_benford(group, "votes_cast")
        n = len(group)
        expected = expected_benford()
        # 95% Wilson confidence intervals for each digit proportion
        ci: dict[int, tuple[float, float]] = {}
        for d in range(1, 10):
            p = observed.get(d, 0.0)
            # Wilson interval
            z = 1.96
            denom = 1 + z**2 / n
            center = (p + z**2 / (2 * n)) / denom
            half = z * math.sqrt(p * (1 - p) / n + z**2 / (4 * n**2)) / denom
            ci[d] = (max(0.0, center - half), min(1.0, center + half))
        chi2_stat, p_val = benford_chi2_test(observed, n)
        result[int(county_id)] = {
            "observed": observed,
            "expected": expected,
            "confidence_intervals": ci,
            "chi2_stat": chi2_stat,
            "p_value": p_val,
            "n_stations": n,
        }
    return result
