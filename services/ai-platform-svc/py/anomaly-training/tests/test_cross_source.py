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
from eel_anomaly_training.cross_source import (
    generate_labelled_anomalies,
    predict_divergence,
    train_xgboost_classifier,
)


def _df():
    loader = KenyaHistoricalDataLoader()
    return loader.load_2013()


def test_labelled_anomalies_has_label_column() -> None:
    df = _df()
    labelled = generate_labelled_anomalies(df)
    assert "label" in labelled.columns
    assert set(labelled["label"].unique()).issubset({0, 1})
    assert labelled["label"].sum() > 0


def test_xgboost_accuracy_above_threshold() -> None:
    df = _df()
    labelled = generate_labelled_anomalies(df)
    _, accuracy = train_xgboost_classifier(labelled)
    assert accuracy >= 0.80, f"accuracy too low: {accuracy:.4f}"


def test_predict_returns_array() -> None:
    df = _df()
    labelled = generate_labelled_anomalies(df)
    model, _ = train_xgboost_classifier(labelled)
    preds = predict_divergence(model, df)
    assert isinstance(preds, np.ndarray)
    assert preds.shape == (len(df),)
    assert set(preds).issubset({0, 1})
