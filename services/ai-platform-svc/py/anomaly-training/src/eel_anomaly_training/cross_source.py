from __future__ import annotations

from typing import Any

import numpy as np
import pandas as pd
from xgboost import XGBClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score


_FEATURE_COLS: list[str] = [
    "turnout_rate",
    "retransmit_count",
    "transmission_delay_secs",
    "rejected_ballots",
    "registered_voters",
    "votes_cast",
    "valid_votes",
]


def generate_labelled_anomalies(df: pd.DataFrame, rng_seed: int = 42) -> pd.DataFrame:
    """Inject synthetic anomalies and return DataFrame with 'label' column.

    label=1 means anomalous, label=0 means normal.
    """
    rng = np.random.default_rng(rng_seed)
    df = df.copy()
    df["label"] = 0

    n = len(df)
    n_anomalies = int(n * 0.10)
    indices = rng.choice(n, size=n_anomalies, replace=False)

    for i, idx in enumerate(indices):
        kind = i % 4
        if kind == 0:
            # extreme turnout
            reg = int(df.at[idx, "registered_voters"])
            df.at[idx, "votes_cast"] = int(reg * rng.uniform(0.99, 1.0))
            df.at[idx, "turnout_rate"] = min(1.0, df.at[idx, "votes_cast"] / reg)
        elif kind == 1:
            # votes_cast > registered_voters (impossible)
            reg = int(df.at[idx, "registered_voters"])
            df.at[idx, "votes_cast"] = reg + int(rng.integers(1, 50))
            df.at[idx, "turnout_rate"] = df.at[idx, "votes_cast"] / reg
        elif kind == 2:
            # huge retransmit count
            df.at[idx, "retransmit_count"] = int(rng.integers(20, 100))
        else:
            # zero valid votes with nonzero votes_cast
            df.at[idx, "valid_votes"] = 0
            df.at[idx, "rejected_ballots"] = int(df.at[idx, "votes_cast"])

        df.at[idx, "label"] = 1

    return df


def train_xgboost_classifier(df: pd.DataFrame) -> tuple[Any, float]:
    """Train XGBoost binary classifier on 'label' column.

    Returns (fitted_model, accuracy_on_test_split).
    """
    feature_cols = [c for c in _FEATURE_COLS if c in df.columns]
    X = df[feature_cols].fillna(0).astype(np.float32).values
    y = df["label"].astype(int).values

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=42, stratify=y
    )

    model = XGBClassifier(
        n_estimators=100,
        max_depth=4,
        learning_rate=0.1,
        eval_metric="logloss",
        random_state=42,
    )
    model.fit(X_train, y_train)
    preds = model.predict(X_test)
    accuracy = float(accuracy_score(y_test, preds))
    return model, accuracy


def predict_divergence(model: Any, df: pd.DataFrame) -> np.ndarray:
    """Return binary predictions (0/1) for each row."""
    feature_cols = [c for c in _FEATURE_COLS if c in df.columns]
    X = df[feature_cols].fillna(0).astype(np.float32).values
    return model.predict(X).astype(int)
