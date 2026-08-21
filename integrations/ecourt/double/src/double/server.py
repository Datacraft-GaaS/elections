"""ecourt double: canned responses for dev and contract tests.

Knobs (env vars):
  ECOURT_LATENCY_MS  - artificial response delay in ms (default 0)
  ECOURT_ERROR_RATE  - fraction [0,1] of requests returning 503 (default 0)
"""

from __future__ import annotations

import asyncio
import os
import random

from fastapi import FastAPI

_LATENCY_MS = int(os.environ.get("ECOURT_LATENCY_MS", "0"))
_ERROR_RATE = float(os.environ.get("ECOURT_ERROR_RATE", "0"))

app = FastAPI(title="eel-adapter-ecourt-double")


@app.post("/ecourt/petition/file")
async def ecourt_petition_file(body: dict) -> dict:
	"File election petition."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}

@app.post("/ecourt/document/upload")
async def ecourt_document_upload(body: dict) -> dict:
	"Upload supporting document."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}

@app.post("/ecourt/case/status")
async def ecourt_case_status(body: dict) -> dict:
	"Check case status."
	if _LATENCY_MS > 0:
		await asyncio.sleep(_LATENCY_MS / 1000)
	if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
		from fastapi import Response as R
		return R(status_code=503, content="simulated error")
	return {"ok": True, "message": "ok", "request_id": body.get("request_id", "test")}


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
