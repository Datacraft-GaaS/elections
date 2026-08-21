"""odpc-breach double: canned responses for dev and contract tests.

Knobs (env vars):
  ODPC_BREACH_LATENCY_MS  - artificial response delay in ms (default 0)
  ODPC_BREACH_ERROR_RATE  - fraction [0,1] of requests returning 503 (default 0)
"""

from __future__ import annotations

import asyncio
import os
import random

from fastapi import FastAPI

_LATENCY_MS = int(os.environ.get("ODPC_BREACH_LATENCY_MS", "0"))
_ERROR_RATE = float(os.environ.get("ODPC_BREACH_ERROR_RATE", "0"))

app = FastAPI(title="eel-adapter-odpc-breach-double")


@app.post("/odpc/breach/notify")
async def odpc_breach_notify(body: dict) -> dict:
	"Submit 72h breach notification."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}

@app.post("/odpc/breach/status")
async def odpc_breach_status(body: dict) -> dict:
	"Check notification status."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
