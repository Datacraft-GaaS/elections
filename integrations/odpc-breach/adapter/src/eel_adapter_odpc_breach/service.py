"""eel-adapter-odpc-breach: FastAPI proxy for ODPC 72h automated breach notification adapter per Kenya DPA section 43."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import httpx
from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import Request, Response

_LOGGER = logging.getLogger(__name__)

_BASE_URL = os.environ.get("ODPC_BREACH_BASE_URL", "https://api.example.go.ke")
_API_KEY = os.environ.get("ODPC_BREACH_API_KEY", "")
_TOKEN_URL = os.environ.get("ODPC_BREACH_TOKEN_URL", "")

_http: httpx.AsyncClient | None = None


def uuid7str() -> str:
	return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
	global _http
	_http = httpx.AsyncClient(timeout=10.0)
	yield
	await _http.aclose()


app = FastAPI(title="eel-adapter-odpc-breach", lifespan=_lifespan)


@app.post(
	"/odpc/breach/notify",
	response_model=Response,
	status_code=status.HTTP_200_OK,
)
async def odpc_breach_notify(req: Request) -> Response:
	"""Submit 72h breach notification."""
	assert _http is not None
	headers: dict[str, str] = {}
	if _API_KEY:
		headers["X-API-Key"] = _API_KEY
	try:
		resp = await _http.post(
			f"{_BASE_URL}/odpc/breach/notify",
			json=req.model_dump(exclude={"request_id"}),
			headers=headers,
		)
		resp.raise_for_status()
		data = resp.json()
		return Response(
			request_id=req.request_id,
			ok=data.get("ok", True),
			message=data.get("message", "ok"),
		)
	except httpx.HTTPStatusError as exc:
		_LOGGER.error("odpc-breach upstream error: %s", exc)
		raise HTTPException(status_code=502, detail="odpc-breach upstream error") from exc


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
