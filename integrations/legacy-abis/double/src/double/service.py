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

"""Legacy ABIS double: returns canned biometric match responses."""

from __future__ import annotations

import asyncio
import os

from fastapi import FastAPI
from pydantic import BaseModel, ConfigDict

_LATENCY_MS = int(os.environ.get("ABIS_LATENCY_MS", "0"))
_MATCH_SCORE = float(os.environ.get("ABIS_MATCH_SCORE", "0.95"))

app = FastAPI(title="eel-adapter-legacy-abis-double")


class _ProbeIn(BaseModel):
    model_config = ConfigDict(extra="allow")
    probe: str = ""
    modality: str = "fingerprint"
    threshold: float = 0.8


class _RefIn(BaseModel):
    model_config = ConfigDict(extra="allow")
    probe: str = ""
    referenceId: str = ""
    modality: str = "fingerprint"


@app.post("/abis/1n")
async def match_1n(body: _ProbeIn) -> dict:
    if _LATENCY_MS > 0:
        await asyncio.sleep(_LATENCY_MS / 1000)
    return {"matched": True, "score": _MATCH_SCORE, "candidateId": "voter-abc-123"}


@app.post("/abis/11")
async def match_11(body: _RefIn) -> dict:
    if _LATENCY_MS > 0:
        await asyncio.sleep(_LATENCY_MS / 1000)
    return {"matched": True, "score": _MATCH_SCORE}


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
