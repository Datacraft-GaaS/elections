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

"""IPRS adapter: FastAPI proxy for IPRS SOAP+OAuth national-ID verification."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import httpx
from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import VerifyRequest, VerifyResult

_LOGGER = logging.getLogger(__name__)

_IPRS_BASE = os.environ.get("IPRS_BASE_URL", "https://iprs.go.ke/api")
_IPRS_TOKEN_URL = os.environ.get("IPRS_TOKEN_URL", "https://iprs.go.ke/oauth/token")
_IPRS_CLIENT_ID = os.environ.get("IPRS_CLIENT_ID", "")
_IPRS_CLIENT_SECRET = os.environ.get("IPRS_CLIENT_SECRET", "")

_http: httpx.AsyncClient | None = None


def uuid7str() -> str:
    return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
    global _http
    _http = httpx.AsyncClient(timeout=10.0)
    yield
    await _http.aclose()


app = FastAPI(title="eel-adapter-iprs", lifespan=_lifespan)


async def _get_token() -> str:
    assert _http is not None
    resp = await _http.post(
        _IPRS_TOKEN_URL,
        data={
            "grant_type": "client_credentials",
            "client_id": _IPRS_CLIENT_ID,
            "client_secret": _IPRS_CLIENT_SECRET,
        },
    )
    resp.raise_for_status()
    return resp.json()["access_token"]


@app.post("/iprs/verify", response_model=VerifyResult, status_code=status.HTTP_200_OK)
async def verify(req: VerifyRequest) -> VerifyResult:
    """Verify a national ID against the IPRS registry."""
    assert _http is not None
    token = await _get_token()
    payload = {"nationalId": req.national_id}
    if req.full_name:
        payload["fullName"] = req.full_name
    if req.dob:
        payload["dateOfBirth"] = req.dob
    try:
        resp = await _http.post(
            f"{_IPRS_BASE}/verify",
            json=payload,
            headers={"Authorization": f"Bearer {token}"},
        )
        resp.raise_for_status()
        data = resp.json()
        return VerifyResult(
            request_id=req.request_id,
            matched=data.get("matched", False),
            duplicate=data.get("duplicate", False),
            status_code=data.get("statusCode", "200"),
            message=data.get("message", "ok"),
        )
    except httpx.HTTPStatusError as exc:
        _LOGGER.error("IPRS upstream error: %s", exc)
        raise HTTPException(status_code=502, detail="IPRS upstream error") from exc


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
