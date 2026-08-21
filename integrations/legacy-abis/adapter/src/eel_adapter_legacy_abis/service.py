"""Legacy ABIS adapter: wraps IEBC vendor SOAP/gRPC into eel-abis schema."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import httpx
from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import Match11Request, Match1NRequest, MatchResult

_LOGGER = logging.getLogger(__name__)

_ABIS_BASE = os.environ.get("ABIS_BASE_URL", "http://localhost:9090")

_http: httpx.AsyncClient | None = None


def uuid7str() -> str:
    return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
    global _http
    _http = httpx.AsyncClient(timeout=30.0)
    yield
    await _http.aclose()


app = FastAPI(title="eel-adapter-legacy-abis", lifespan=_lifespan)


@app.post("/abis/match/1n", response_model=MatchResult, status_code=status.HTTP_200_OK)
async def match_1n(req: Match1NRequest) -> MatchResult:
    """1:N biometric search against the voter database."""
    assert _http is not None
    try:
        resp = await _http.post(
            f"{_ABIS_BASE}/abis/1n",
            json={
                "probe": req.probe_template,
                "modality": req.modality,
                "threshold": req.threshold,
            },
        )
        resp.raise_for_status()
        data = resp.json()
        return MatchResult(
            request_id=req.request_id,
            matched=data.get("matched", False),
            score=data.get("score", 0.0),
            candidate_id=data.get("candidateId"),
        )
    except httpx.HTTPStatusError as exc:
        _LOGGER.error("ABIS 1N error: %s", exc)
        raise HTTPException(status_code=502, detail="ABIS upstream error") from exc


@app.post("/abis/match/11", response_model=MatchResult, status_code=status.HTTP_200_OK)
async def match_11(req: Match11Request) -> MatchResult:
    """1:1 biometric verification against a registered voter."""
    assert _http is not None
    try:
        resp = await _http.post(
            f"{_ABIS_BASE}/abis/11",
            json={
                "probe": req.probe_template,
                "referenceId": req.reference_id,
                "modality": req.modality,
            },
        )
        resp.raise_for_status()
        data = resp.json()
        return MatchResult(
            request_id=req.request_id,
            matched=data.get("matched", False),
            score=data.get("score", 0.0),
            candidate_id=req.reference_id if data.get("matched") else None,
        )
    except httpx.HTTPStatusError as exc:
        _LOGGER.error("ABIS 11 error: %s", exc)
        raise HTTPException(status_code=502, detail="ABIS upstream error") from exc


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
