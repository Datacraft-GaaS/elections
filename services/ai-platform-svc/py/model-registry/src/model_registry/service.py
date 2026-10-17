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

import hashlib
import logging
from datetime import datetime, timedelta, timezone

from fastapi import FastAPI, HTTPException
from uuid6 import uuid7

from model_registry.views import (
    DecisionLogEntry,
    DecisionLogRequest,
    ModelActivateRequest,
    ModelCard,
    ModelRegistrationRequest,
    ModelStatus,
)

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-model-registry", version="0.1.0")

models_store: dict[str, ModelCard] = {}
decision_log_store: list[DecisionLogEntry] = []
pii_events_store: list[dict] = []


def now() -> datetime:
    return datetime.now(timezone.utc)


def uuid7str() -> str:
    return str(uuid7())


def _log_register(model_id: str) -> None:
    logger.info("eel.ai.model_card_published.v1 model_id=%s", model_id)


def _log_activate(model_id: str) -> None:
    logger.info("eel.ai.model_activated.v1 model_id=%s", model_id)


@app.get("/v1/ai/registry/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


@app.get("/v1/ai/registry/models")
async def list_models() -> list[ModelCard]:
    return list(models_store.values())


@app.get("/v1/ai/registry/models/{model_id}")
async def get_model(model_id: str) -> ModelCard:
    model = models_store.get(model_id)
    if model is None:
        raise HTTPException(status_code=404, detail="model not found")
    return model


@app.get("/v1/ai/registry/models/{model_id}/card")
async def get_model_card(model_id: str) -> dict:
    model = models_store.get(model_id)
    if model is None:
        raise HTTPException(status_code=404, detail="model not found")
    raw = model.model_dump_json()
    card_hash = hashlib.sha256(raw.encode()).hexdigest()
    markdown = f"# {model.name} v{model.version}\n\n{model.training_data_description}\n\n**Accuracy**: {model.eval_accuracy}\n"
    return {"card_hash": card_hash, "markdown": markdown}


@app.post("/v1/ai/registry/models", status_code=201)
async def register_model(req: ModelRegistrationRequest) -> ModelCard:
    model_id = uuid7str()
    registered = now()
    status = ModelStatus.REGISTERED
    if req.iab_notified_at is None:
        status = ModelStatus.PENDING_IAB
    card = ModelCard(
        id=model_id,
        code=req.code,
        name=req.name,
        version=req.version,
        layer=req.layer,
        architecture=req.architecture,
        training_data_description=req.training_data_description,
        eval_accuracy=req.eval_accuracy,
        eval_bias_flag=req.eval_bias_flag,
        known_limitations=req.known_limitations,
        human_in_loop=req.human_in_loop,
        sovereign_hosting=req.sovereign_hosting,
        model_card_url=req.model_card_url,
        iab_notified_at=req.iab_notified_at,
        ccb_approval_id=req.ccb_approval_id,
        deployed_at=req.deployed_at,
        rollback_procedure=req.rollback_procedure,
        is_active=False,
        status=status,
        registered_at=registered,
    )
    models_store[model_id] = card
    _log_register(model_id)
    return card


@app.patch("/v1/ai/registry/models/{model_id}/activate")
async def activate_model(model_id: str, req: ModelActivateRequest) -> ModelCard:
    model = models_store.get(model_id)
    if model is None:
        raise HTTPException(status_code=404, detail="model not found")
    if model.iab_notified_at is None:
        raise HTTPException(status_code=422, detail="IAB notification required before activation")
    elapsed = now() - model.iab_notified_at
    if elapsed < timedelta(hours=72):
        raise HTTPException(status_code=422, detail="IAB 72-hour waiting period not elapsed")
    updated = model.model_copy(
        update={
            "ccb_approval_id": req.ccb_approval_id,
            "is_active": True,
            "deployed_at": now(),
            "status": ModelStatus.ACTIVE,
        }
    )
    models_store[model_id] = updated
    _log_activate(model_id)
    return updated


@app.get("/v1/ai/registry/decision-log")
async def list_decision_log() -> list[DecisionLogEntry]:
    return decision_log_store


@app.post("/v1/ai/registry/decision-log", status_code=201)
async def create_decision_log_entry(req: DecisionLogRequest) -> DecisionLogEntry:
    input_hash = hashlib.sha256(req.input_hash.encode()).hexdigest()
    entry = DecisionLogEntry(
        id=uuid7str(),
        model_id=req.model_id,
        model_version=req.model_version,
        interaction_type=req.interaction_type,
        input_hash=input_hash,
        confidence_score=req.confidence_score,
        model_output=req.model_output,
        human_decision=req.human_decision,
        human_actor_id=req.human_actor_id,
        human_decision_at=req.human_decision_at,
        decision_at=now(),
    )
    decision_log_store.append(entry)
    return entry
