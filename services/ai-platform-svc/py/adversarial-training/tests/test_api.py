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
