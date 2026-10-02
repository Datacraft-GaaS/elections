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
from datetime import datetime, timezone
from enum import Enum
from typing import Any

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, ConfigDict
from uuid6 import uuid7


def _uuid7str() -> str:
    return str(uuid7())


app = FastAPI(title="eel-anomaly-training", version="0.1.0")


class ModelTypeEnum(str, Enum):
    isolation_forest = "isolation_forest"
    xgboost = "xgboost"
    benford = "benford"


class RunStatus(str, Enum):
    pending = "pending"
    running = "running"
    completed = "completed"
    failed = "failed"
    promoted = "promoted"


class TrainRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    election_years: list[int] = [2013, 2017, 2022]
    notes: str | None = None


class TrainResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    run_id: str
    status: RunStatus


class RunResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    run_id: str
    model_type: ModelTypeEnum
    status: RunStatus
    started_at: str
    completed_at: str | None = None
    accuracy: float | None = None
    error: str | None = None
    promoted: bool = False


class PromoteResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    run_id: str
    promoted: bool
    promoted_at: str


# In-memory run storage
_runs: dict[str, RunResult] = {}


async def _run_training(run_id: str, model_type: ModelTypeEnum) -> None:
    """Background training task."""
    run = _runs[run_id]
    _runs[run_id] = run.model_copy(update={"status": RunStatus.running})
    await asyncio.sleep(0)  # yield to event loop

    try:
        from eel_anomaly_training.loader import KenyaHistoricalDataLoader

        loader = KenyaHistoricalDataLoader()
        df = loader.load_all()
        accuracy: float | None = None

        if model_type == ModelTypeEnum.isolation_forest:
            from eel_anomaly_training.isolation_forest import train_isolation_forest

            train_isolation_forest(df)

        elif model_type == ModelTypeEnum.xgboost:
            from eel_anomaly_training.cross_source import (
                generate_labelled_anomalies,
                train_xgboost_classifier,
            )

            labelled = generate_labelled_anomalies(df)
            _, accuracy = train_xgboost_classifier(labelled)

        elif model_type == ModelTypeEnum.benford:
            from eel_anomaly_training.benford import fit_county_benford

            fit_county_benford(df)

        _runs[run_id] = _runs[run_id].model_copy(
            update={
                "status": RunStatus.completed,
                "completed_at": datetime.now(tz=timezone.utc).isoformat(),
                "accuracy": accuracy,
            }
        )

    except Exception as exc:
        _runs[run_id] = _runs[run_id].model_copy(
            update={
                "status": RunStatus.failed,
                "completed_at": datetime.now(tz=timezone.utc).isoformat(),
                "error": str(exc),
            }
        )


@app.post("/training/{model_type}/train", response_model=TrainResponse)
async def train(model_type: ModelTypeEnum, body: TrainRequest) -> TrainResponse:
    run_id = _uuid7str()
    now = datetime.now(tz=timezone.utc).isoformat()
    run = RunResult(
        run_id=run_id,
        model_type=model_type,
        status=RunStatus.pending,
        started_at=now,
    )
    _runs[run_id] = run
    asyncio.create_task(_run_training(run_id, model_type))
    return TrainResponse(run_id=run_id, status=RunStatus.pending)


@app.get("/training/{run_id}", response_model=RunResult)
async def get_run(run_id: str) -> RunResult:
    if run_id not in _runs:
        raise HTTPException(status_code=404, detail="run not found")
    return _runs[run_id]


@app.post("/training/{run_id}/promote", response_model=PromoteResponse)
async def promote(run_id: str) -> PromoteResponse:
    if run_id not in _runs:
        raise HTTPException(status_code=404, detail="run not found")
    run = _runs[run_id]
    if run.status != RunStatus.completed:
        raise HTTPException(
            status_code=400, detail=f"run status is {run.status}, must be completed"
        )
    now = datetime.now(tz=timezone.utc).isoformat()
    _runs[run_id] = run.model_copy(
        update={"status": RunStatus.promoted, "promoted": True}
    )
    return PromoteResponse(run_id=run_id, promoted=True, promoted_at=now)
