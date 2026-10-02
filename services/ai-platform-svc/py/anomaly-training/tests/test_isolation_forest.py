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
import pytest

from eel_anomaly_training.loader import KenyaHistoricalDataLoader
from eel_anomaly_training.isolation_forest import (
    compute_anomaly_scores,
    train_isolation_forest,
)


def _small_df():
    loader = KenyaHistoricalDataLoader()
    return loader.load_2013().head(500)


def test_train_returns_model_and_onnx_bytes() -> None:
    df = _small_df()
    model, onnx_bytes = train_isolation_forest(df)
    assert model is not None
    assert isinstance(onnx_bytes, bytes)
    assert len(onnx_bytes) > 0


def test_onnx_round_trip() -> None:
    import onnxruntime as rt

    df = _small_df()
    _, onnx_bytes = train_isolation_forest(df)

    sess = rt.InferenceSession(onnx_bytes)
    input_name = sess.get_inputs()[0].name

    sample = df.head(10)
    votes_cast = sample["votes_cast"].clip(lower=1)
    valid_votes = sample["valid_votes"].clip(lower=1)
    sample = sample.copy()
    sample["rejected_ballot_rate"] = sample["rejected_ballots"] / votes_cast
    candidate_cols = [f"candidate_{i}_votes" for i in range(1, 6)]
    candidate_max = sample[candidate_cols].max(axis=1)
    sample["vote_concentration_index"] = candidate_max / valid_votes

    features = [
        "turnout_rate",
        "retransmit_count",
        "transmission_delay_secs",
        "rejected_ballot_rate",
        "vote_concentration_index",
    ]
    X = sample[features].fillna(0).astype(np.float32).values
    outputs = sess.run(None, {input_name: X})
    assert outputs is not None
    assert len(outputs) > 0


def test_anomaly_scores_shape() -> None:
    df = _small_df()
    model, _ = train_isolation_forest(df)
    scores = compute_anomaly_scores(model, df)
    assert scores.shape == (len(df),)


def test_anomaly_scores_range() -> None:
    df = _small_df()
    model, _ = train_isolation_forest(df)
    scores = compute_anomaly_scores(model, df)
    # IsolationForest score_samples returns values roughly in [-1, 1]
    # More precisely in (offset - 1, offset] but practically within [-1, 1]
    assert scores.min() >= -1.0 - 1e-6
    assert scores.max() <= 1.0 + 1e-6
