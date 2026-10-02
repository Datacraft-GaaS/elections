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

"""kps-kdf-unhcr double: canned responses for dev and contract tests.

Knobs (env vars):
  KPS_KDF_UNHCR_LATENCY_MS  - artificial response delay in ms (default 0)
  KPS_KDF_UNHCR_ERROR_RATE  - fraction [0,1] of requests returning 503 (default 0)
"""

from __future__ import annotations

import asyncio
import os
import random

from fastapi import FastAPI

_LATENCY_MS = int(os.environ.get("KPS_KDF_UNHCR_LATENCY_MS", "0"))
_ERROR_RATE = float(os.environ.get("KPS_KDF_UNHCR_ERROR_RATE", "0"))

app = FastAPI(title="eel-adapter-kps-kdf-unhcr-double")


@app.post("/displaced/register")
async def displaced_register(body: dict) -> dict:
	"Register displaced voter."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}

@app.post("/displaced/status")
async def displaced_status(body: dict) -> dict:
	"Check displaced voter status."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
