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
from sklearn.ensemble import IsolationForest
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType


_FEATURES: list[str] = [
    "turnout_rate",
    "retransmit_count",
    "transmission_delay_secs",
    "rejected_ballot_rate",
    "vote_concentration_index",
]


def _build_feature_matrix(df: pd.DataFrame) -> np.ndarray:
    df = df.copy()
    votes_cast = df["votes_cast"].clip(lower=1)
    valid_votes = df["valid_votes"].clip(lower=1)

    df["rejected_ballot_rate"] = df["rejected_ballots"] / votes_cast
    candidate_cols = [f"candidate_{i}_votes" for i in range(1, 6)]
    candidate_max = df[candidate_cols].max(axis=1)
    df["vote_concentration_index"] = candidate_max / valid_votes

    return df[_FEATURES].fillna(0).astype(np.float32).values


def train_isolation_forest(df: pd.DataFrame) -> tuple[IsolationForest, bytes]:
    """Train an IsolationForest on station-level features.

    Returns the fitted model and its ONNX serialization.
    """
    X = _build_feature_matrix(df)
    model = IsolationForest(n_estimators=100, contamination=0.05, random_state=42)
    model.fit(X)

    initial_type = [("float_input", FloatTensorType([None, X.shape[1]]))]
    onnx_model = convert_sklearn(
        model,
        initial_types=initial_type,
        target_opset={"": 17, "ai.onnx.ml": 3},
    )
    onnx_bytes = onnx_model.SerializeToString()

    return model, onnx_bytes


def compute_anomaly_scores(model: IsolationForest, df: pd.DataFrame) -> np.ndarray:
    """Return anomaly scores in [-1, 1] (negative = more anomalous)."""
    X = _build_feature_matrix(df)
    return model.score_samples(X)
