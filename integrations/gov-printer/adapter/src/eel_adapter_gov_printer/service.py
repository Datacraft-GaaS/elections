"""eel-adapter-gov-printer: FastAPI proxy for Government printer Gazette submission adapter for M52 and M40."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import httpx
from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import Request, Response

_LOGGER = logging.getLogger(__name__)

_BASE_URL = os.environ.get("GOV_PRINTER_BASE_URL", "https://api.example.go.ke")
_API_KEY = os.environ.get("GOV_PRINTER_API_KEY", "")
_TOKEN_URL = os.environ.get("GOV_PRINTER_TOKEN_URL", "")

_http: httpx.AsyncClient | None = None


def uuid7str() -> str:
	return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
	global _http
	_http = httpx.AsyncClient(timeout=10.0)
	yield
	await _http.aclose()


app = FastAPI(title="eel-adapter-gov-printer", lifespan=_lifespan)


@app.post(
	"/gov-printer/gazette/submit",
	response_model=Response,
	status_code=status.HTTP_200_OK,
)
async def gov_printer_gazette_submit(req: Request) -> Response:
	"""Submit gazette document."""
	assert _http is not None
	headers: dict[str, str] = {}
	if _API_KEY:
		headers["X-API-Key"] = _API_KEY
	try:
		resp = await _http.post(
			f"{_BASE_URL}/gov-printer/gazette/submit",
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
		_LOGGER.error("gov-printer upstream error: %s", exc)
		raise HTTPException(status_code=502, detail="gov-printer upstream error") from exc


@app.get("/health")
async def health() -> dict[str, str]:
	return {"status": "ok"}
