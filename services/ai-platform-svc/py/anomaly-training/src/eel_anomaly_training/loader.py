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
import pandas as pd

# Kenya county population weights (approximate, proportional to registered voters)
_COUNTY_WEIGHTS: list[float] = [
    0.08,
    0.05,
    0.04,
    0.03,
    0.02,
    0.02,
    0.03,
    0.02,
    0.02,
    0.03,  # 1-10
    0.03,
    0.02,
    0.03,
    0.04,
    0.02,
    0.02,
    0.03,
    0.02,
    0.03,
    0.02,  # 11-20
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.03,
    0.02,
    0.02,  # 21-30
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,  # 31-40
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,
    0.02,  # 41-47
]

# Total stations: 46229 across 47 counties
_TOTAL_STATIONS = 46_229
_NUM_COUNTIES = 47


def _stations_per_county() -> list[int]:
    weights = np.array(_COUNTY_WEIGHTS, dtype=float)
    weights = weights / weights.sum()
    raw = (weights * _TOTAL_STATIONS).astype(int)
    diff = _TOTAL_STATIONS - raw.sum()
    # distribute remainder to largest counties
    order = np.argsort(-weights)
    for i in range(diff):
        raw[order[i % len(order)]] += 1
    return raw.tolist()


def _generate_year(year: int, rng: np.random.Generator) -> pd.DataFrame:
    spc = _stations_per_county()
    rows: list[dict] = []

    for county_idx, n_stations in enumerate(spc):
        county_id = county_idx + 1
        # turnout varies by county: some counties lower
        base_turnout = rng.uniform(0.60, 0.82)

        for station_idx in range(n_stations):
            station_id = f"KE-{county_id:02d}-{station_idx + 1:04d}"
            registered = int(rng.integers(200, 3001))
            turnout = float(np.clip(rng.normal(base_turnout, 0.05), 0.30, 0.97))
            votes_cast = int(registered * turnout)
            rejected = int(max(0, rng.integers(0, max(1, int(votes_cast * 0.03)))))
            valid = votes_cast - rejected

            # distribute valid votes among 5 candidates
            shares = rng.dirichlet(np.ones(5) * 2)
            candidate_votes = (shares * valid).astype(int)
            candidate_votes[-1] = valid - candidate_votes[:-1].sum()

            delay = int(rng.integers(10, 3601))
            retransmit = int(rng.integers(0, 4))

            rows.append(
                {
                    "station_id": station_id,
                    "county_id": county_id,
                    "registered_voters": registered,
                    "votes_cast": votes_cast,
                    "valid_votes": valid,
                    "rejected_ballots": rejected,
                    "candidate_1_votes": int(candidate_votes[0]),
                    "candidate_2_votes": int(candidate_votes[1]),
                    "candidate_3_votes": int(candidate_votes[2]),
                    "candidate_4_votes": int(candidate_votes[3]),
                    "candidate_5_votes": int(candidate_votes[4]),
                    "turnout_rate": round(float(votes_cast) / float(registered), 6),
                    "transmission_delay_secs": delay,
                    "retransmit_count": retransmit,
                    "election_year": year,
                }
            )

    df = pd.DataFrame(rows)
    df = _inject_anomalies(df, rng)
    return df


def _inject_anomalies(df: pd.DataFrame, rng: np.random.Generator) -> pd.DataFrame:
    n = len(df)
    anomaly_frac = 0.025  # 2.5% anomalous
    n_anomalies = int(n * anomaly_frac)
    indices = rng.choice(n, size=n_anomalies, replace=False)

    df = df.copy()
    for i, idx in enumerate(indices):
        kind = i % 3
        if kind == 0:
            # turnout > 98%
            reg = int(df.at[idx, "registered_voters"])
            votes = int(reg * rng.uniform(0.985, 1.001))
            votes = min(votes, reg)
            df.at[idx, "votes_cast"] = votes
            df.at[idx, "turnout_rate"] = round(float(votes) / float(reg), 6)
        elif kind == 1:
            # candidate concentration > 95%
            valid = int(df.at[idx, "valid_votes"])
            dominant = int(valid * rng.uniform(0.95, 0.99))
            rest = valid - dominant
            df.at[idx, "candidate_1_votes"] = dominant
            df.at[idx, "candidate_2_votes"] = rest
            df.at[idx, "candidate_3_votes"] = 0
            df.at[idx, "candidate_4_votes"] = 0
            df.at[idx, "candidate_5_votes"] = 0
        else:
            # mismatched vote counts: valid_votes > votes_cast
            votes_cast = int(df.at[idx, "votes_cast"])
            df.at[idx, "valid_votes"] = votes_cast + int(rng.integers(10, 100))

    return df


class KenyaHistoricalDataLoader:
    """Loads Kenya election data for 2013, 2017, 2022.

    When raw CSVs are unavailable (always in CI), generates a deterministic
    synthetic corpus using numpy.random.default_rng(seed=42).
    """

    def __init__(self, data_dir: str | None = None) -> None:
        self._data_dir = data_dir
        self._rng_seed = 42

    def _rng(self) -> np.random.Generator:
        return np.random.default_rng(self._rng_seed)

    def load_2013(self) -> pd.DataFrame:
        rng = self._rng()
        return _generate_year(2013, rng)

    def load_2017(self) -> pd.DataFrame:
        rng = np.random.default_rng(self._rng_seed + 1)
        return _generate_year(2017, rng)

    def load_2022(self) -> pd.DataFrame:
        rng = np.random.default_rng(self._rng_seed + 2)
        return _generate_year(2022, rng)

    def load_all(self) -> pd.DataFrame:
        frames = [self.load_2013(), self.load_2017(), self.load_2022()]
        return pd.concat(frames, ignore_index=True)
