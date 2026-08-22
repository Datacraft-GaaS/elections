from __future__ import annotations

import pandas as pd
import pytest

from eel_anomaly_training.loader import KenyaHistoricalDataLoader

_EXPECTED_STATIONS = 46_229
_EXPECTED_COLUMNS = [
    "station_id",
    "county_id",
    "registered_voters",
    "votes_cast",
    "valid_votes",
    "rejected_ballots",
    "candidate_1_votes",
    "candidate_2_votes",
    "candidate_3_votes",
    "candidate_4_votes",
    "candidate_5_votes",
    "turnout_rate",
    "transmission_delay_secs",
    "retransmit_count",
    "election_year",
]


def test_synthetic_corpus_row_count() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_all()
    assert len(df) == _EXPECTED_STATIONS * 3


def test_synthetic_corpus_deterministic() -> None:
    loader = KenyaHistoricalDataLoader()
    df1 = loader.load_all()
    df2 = loader.load_all()
    pd.testing.assert_frame_equal(
        df1.reset_index(drop=True), df2.reset_index(drop=True)
    )


def test_schema_columns() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_2013()
    for col in _EXPECTED_COLUMNS:
        assert col in df.columns, f"missing column: {col}"


def test_county_range() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_all()
    assert df["county_id"].min() >= 1
    assert df["county_id"].max() <= 47


def test_anomaly_injection() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_all()
    # anomalies: turnout > 0.98 OR valid_votes > votes_cast OR county max candidate share > 0.95
    high_turnout = df["turnout_rate"] > 0.98
    mismatch = df["valid_votes"] > df["votes_cast"]
    candidate_cols = [f"candidate_{i}_votes" for i in range(1, 6)]
    candidate_max = df[candidate_cols].max(axis=1)
    concentration = candidate_max / df["valid_votes"].clip(lower=1)
    high_conc = concentration > 0.94

    anomalous = high_turnout | mismatch | high_conc
    frac = anomalous.sum() / len(df)
    assert frac >= 0.01, f"anomaly fraction too low: {frac:.4f}"


def test_load_2013_shape() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_2013()
    assert len(df) == _EXPECTED_STATIONS


def test_load_2017_shape() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_2017()
    assert len(df) == _EXPECTED_STATIONS


def test_load_2022_shape() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_2022()
    assert len(df) == _EXPECTED_STATIONS


def test_election_year_values() -> None:
    loader = KenyaHistoricalDataLoader()
    df = loader.load_all()
    assert set(df["election_year"].unique()) == {2013, 2017, 2022}
