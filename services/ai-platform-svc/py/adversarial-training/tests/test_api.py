from __future__ import annotations

import asyncio

from fastapi.testclient import TestClient

from eel_adversarial_training.api import app

client = TestClient(app)


def test_train_returns_run_id() -> None:
    resp = client.post(
        "/adversarial/train",
        json={"model_type": "binary_linear", "dataset": "faceforensics", "config": {}},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "run_id" in data
    assert isinstance(data["run_id"], str)
    assert len(data["run_id"]) > 0
    assert data["status"] == "ok"


def test_robustness_report_format() -> None:
    # Trigger a training run first so the report is non-empty.
    client.post(
        "/adversarial/train",
        json={"model_type": "logistic", "dataset": "dfdc", "config": {}},
    )
    resp = client.get("/adversarial/robustness-report")
    assert resp.status_code == 200
    data = resp.json()
    assert "run_id" in data
    assert "metrics" in data
    assert "model_type" in data
    assert "dataset" in data


def test_train_unknown_model_type() -> None:
    resp = client.post(
        "/adversarial/train",
        json={"model_type": "not_a_model", "dataset": "faceforensics", "config": {}},
    )
    assert resp.status_code == 422
