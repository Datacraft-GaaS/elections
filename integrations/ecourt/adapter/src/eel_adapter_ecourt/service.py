"""eel-adapter-ecourt: FastAPI proxy for eCourt petition filing and document handoff adapter for M40."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import httpx
from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import Request, Response

_LOGGER = logging.getLogger(__name__)

_BASE_URL = os.environ.get("ECOURT_BASE_URL", "https://api.example.go.ke")
_API_KEY = os.environ.get("ECOURT_API_KEY", "")
_TOKEN_URL = os.environ.get("ECOURT_TOKEN_URL", "")

_http: httpx.AsyncClient | None = None


def uuid7str() -> str:
	return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
	global _http
	_http = httpx.AsyncClient(timeout=10.0)
	yield
	await _http.aclose()


app = FastAPI(title="eel-adapter-ecourt", lifespan=_lifespan)


@app.post(
	"/ecourt/petition/file",
	response_model=Response,
	status_code=status.HTTP_200_OK,
)
async def ecourt_petition_file(req: Request) -> Response:
	"""File election petition."""
	assert _http is not None
	headers: dict[str, str] = {}
	if _API_KEY:
		headers["X-API-Key"] = _API_KEY
	try:
		resp = await _http.post(
			f"{_BASE_URL}/ecourt/petition/file",
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
		_LOGGER.error("ecourt upstream error: %s", exc)
		raise HTTPException(status_code=502, detail="ecourt upstream error") from exc


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
