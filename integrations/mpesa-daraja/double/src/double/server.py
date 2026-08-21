"""mpesa-daraja double: canned responses for dev and contract tests.

Knobs (env vars):
  MPESA_DARAJA_LATENCY_MS  - artificial response delay in ms (default 0)
  MPESA_DARAJA_ERROR_RATE  - fraction [0,1] of requests returning 503 (default 0)
"""

from __future__ import annotations

import asyncio
import os
import random

from fastapi import FastAPI

_LATENCY_MS = int(os.environ.get("MPESA_DARAJA_LATENCY_MS", "0"))
_ERROR_RATE = float(os.environ.get("MPESA_DARAJA_ERROR_RATE", "0"))

app = FastAPI(title="eel-adapter-mpesa-daraja-double")


@app.post("/mpesa/c2b/register")
async def mpesa_c2b_register(body: dict) -> dict:
	"Register C2B URL."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}

@app.post("/mpesa/c2b/callback")
async def mpesa_c2b_callback(body: dict) -> dict:
	"Receive payment notification."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}

@app.post("/mpesa/b2c/pay")
async def mpesa_b2c_pay(body: dict) -> dict:
	"Initiate B2C payment."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
