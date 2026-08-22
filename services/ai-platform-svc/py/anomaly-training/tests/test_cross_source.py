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
