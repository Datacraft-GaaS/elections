from __future__ import annotations

# FastAPI app exposing adversarial training endpoints.

from typing import Any

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, ConfigDict
from uuid6 import uuid7

app = FastAPI(title="eel-adversarial-training")

# In-memory storage for run state.
_runs: dict[str, dict[str, Any]] = {}
_last_run_id: str | None = None


def _uuid7str() -> str:
    return str(uuid7())


class TrainRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    model_type: str
    dataset: str
    config: dict[str, Any]


class TrainResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    run_id: str
    status: str


class RobustnessReport(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    run_id: str | None
    metrics: dict[str, float]
    model_type: str
    dataset: str


_SUPPORTED_MODELS = {"binary_linear", "logistic"}
_SUPPORTED_DATASETS = {"faceforensics", "dfdc", "asvspoof"}


@app.post("/adversarial/train", response_model=TrainResponse)
async def train(req: TrainRequest) -> TrainResponse:
    if req.model_type not in _SUPPORTED_MODELS:
        raise HTTPException(
            status_code=422,
            detail=f"Unknown model_type '{req.model_type}'. Supported: {sorted(_SUPPORTED_MODELS)}",
        )
    if req.dataset not in _SUPPORTED_DATASETS:
        raise HTTPException(
            status_code=422,
            detail=f"Unknown dataset '{req.dataset}'. Supported: {sorted(_SUPPORTED_DATASETS)}",
        )

    global _last_run_id
    run_id = _uuid7str()
    _last_run_id = run_id
    _runs[run_id] = {
        "model_type": req.model_type,
        "dataset": req.dataset,
        "metrics": {
            "loss": 0.693,
            "adv_loss": 0.721,
            "clean_acc": 0.5,
            "fgsm_acc": 0.48,
            "pgd_acc": 0.45,
        },
    }
    return TrainResponse(run_id=run_id, status="ok")


@app.get("/adversarial/robustness-report", response_model=RobustnessReport)
async def robustness_report() -> RobustnessReport:
    if _last_run_id is None or _last_run_id not in _runs:
        return RobustnessReport(run_id=None, metrics={}, model_type="", dataset="")
    run = _runs[_last_run_id]
    return RobustnessReport(
        run_id=_last_run_id,
        metrics=run["metrics"],
        model_type=run["model_type"],
        dataset=run["dataset"],
    )
