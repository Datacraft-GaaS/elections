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


import pytest
from httpx import ASGITransport, AsyncClient

from eel_anomaly_training.api import app, _runs


@pytest.fixture(autouse=True)
def clear_runs():
    _runs.clear()
    yield
    _runs.clear()


async def test_train_returns_run_id() -> None:
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/training/benford/train",
            json={"election_years": [2013]},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert "run_id" in data
    assert data["status"] == "pending"


async def test_get_run_status() -> None:
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/training/benford/train",
            json={"election_years": [2013]},
        )
        run_id = resp.json()["run_id"]

        status_resp = await client.get(f"/training/{run_id}")
    assert status_resp.status_code == 200
    data = status_resp.json()
    assert data["run_id"] == run_id
    assert data["model_type"] == "benford"
    assert data["status"] in ("pending", "running", "completed", "failed")


async def test_get_run_404_unknown() -> None:
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.get("/training/nonexistent-run-id")
    assert resp.status_code == 404


async def test_promote_run() -> None:
    # Manually insert a completed run to avoid waiting for real training
    from eel_anomaly_training.api import RunResult, RunStatus, ModelTypeEnum
    from datetime import datetime, timezone

    run_id = "test-promote-run-id"
    _runs[run_id] = RunResult(
        run_id=run_id,
        model_type=ModelTypeEnum.benford,
        status=RunStatus.completed,
        started_at=datetime.now(tz=timezone.utc).isoformat(),
        completed_at=datetime.now(tz=timezone.utc).isoformat(),
    )

    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(f"/training/{run_id}/promote")

    assert resp.status_code == 200
    data = resp.json()
    assert data["promoted"] is True
    assert data["run_id"] == run_id
    assert "promoted_at" in data


async def test_promote_non_completed_run_fails() -> None:
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/training/benford/train",
            json={"election_years": [2013]},
        )
        run_id = resp.json()["run_id"]
        promote_resp = await client.post(f"/training/{run_id}/promote")

    # pending/running runs cannot be promoted
    assert promote_resp.status_code == 400
