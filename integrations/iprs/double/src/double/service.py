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

"""IPRS double: replays canned IPRS responses for dev and contract tests.

Knobs (env vars):
  IPRS_LATENCY_MS    - artificial response delay in ms (default 0)
  IPRS_ERROR_RATE    - fraction [0,1] of requests that return 503 (default 0)
  IPRS_DUPLICATE_MODE - "true" makes every response set duplicate=true (EC-DAT-15)
"""

from __future__ import annotations

import asyncio
import os
import random

from fastapi import FastAPI
from pydantic import BaseModel, ConfigDict

_LATENCY_MS = int(os.environ.get("IPRS_LATENCY_MS", "0"))
_ERROR_RATE = float(os.environ.get("IPRS_ERROR_RATE", "0"))
_DUPLICATE_MODE = os.environ.get("IPRS_DUPLICATE_MODE", "false").lower() == "true"

app = FastAPI(title="eel-adapter-iprs-double")


class _VerifyIn(BaseModel):
    model_config = ConfigDict(extra="allow")
    nationalId: str
    fullName: str | None = None
    dateOfBirth: str | None = None


class _TokenResp(BaseModel):
    access_token: str = "test-token"
    token_type: str = "Bearer"
    expires_in: int = 3600


@app.post("/oauth/token")
async def token() -> _TokenResp:
    return _TokenResp()


@app.post("/api/verify")
async def verify(body: _VerifyIn) -> dict:
    if _LATENCY_MS > 0:
        await asyncio.sleep(_LATENCY_MS / 1000)
    if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
        from fastapi import Response

        return Response(status_code=503, content="simulated error")
    return {
        "matched": True,
        "duplicate": _DUPLICATE_MODE,
        "statusCode": "200",
        "message": "ok",
    }


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
