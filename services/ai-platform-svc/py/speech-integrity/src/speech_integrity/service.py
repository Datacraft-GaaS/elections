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

import logging
from datetime import datetime, timezone

from fastapi import FastAPI, HTTPException

from speech_integrity.detectors import classify_content, detect_cib, detect_deepfake
from speech_integrity.views import (
    AnalystDecision,
    CibCluster,
    ContentFlag,
    ContentItem,
    DeepfakeResult,
)

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-speech-integrity", version="0.1.0")

flags_store: dict[str, ContentFlag] = {}


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _log_classify(count: int) -> None:
    logger.info("eel.speech.classify.v1 flags=%d", count)


def _log_decision(flag_id: str, decision: str) -> None:
    logger.info("eel.speech.decision.v1 flag_id=%s decision=%s", flag_id, decision)


@app.get("/v1/speech/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


@app.post("/v1/speech/classify")
async def classify(items: list[ContentItem]) -> list[ContentFlag]:
    all_flags: list[ContentFlag] = []
    for item in items:
        flags = await classify_content(item)
        for f in flags:
            flags_store[f.flag_id] = f
        all_flags.extend(flags)
    _log_classify(len(all_flags))
    return all_flags


@app.post("/v1/speech/cib-detect")
async def cib_detect(items: list[ContentItem]) -> list[CibCluster]:
    return await detect_cib(items)


@app.post("/v1/speech/deepfake")
async def deepfake(item: ContentItem) -> DeepfakeResult:
    return await detect_deepfake(item)


@app.patch("/v1/speech/flags/{flag_id}/decision")
async def update_decision(flag_id: str, body: AnalystDecision) -> ContentFlag:
    flag = flags_store.get(flag_id)
    if flag is None:
        raise HTTPException(status_code=404, detail="flag not found")
    if body.flag_id != flag_id:
        raise HTTPException(status_code=400, detail="flag_id mismatch")
    updated = flag.model_copy(
        update={
            "analyst_decision": body.decision,
            "decision_at": _now(),
            "decision_by": body.decision_by,
        }
    )
    flags_store[flag_id] = updated
    _log_decision(flag_id, body.decision)
    return updated


@app.get("/v1/speech/flags")
async def list_flags() -> list[ContentFlag]:
    return list(flags_store.values())
