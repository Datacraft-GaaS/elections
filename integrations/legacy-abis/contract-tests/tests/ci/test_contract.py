"""Legacy ABIS contract tests against the double."""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("ABIS_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_match_1n() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/abis/1n",
            json={"probe": "AQID", "modality": "fingerprint", "threshold": 0.8},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert data["matched"] is True
    assert "score" in data


@pytest.mark.asyncio
async def test_match_11() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/abis/11",
            json={
                "probe": "AQID",
                "referenceId": "voter-001",
                "modality": "fingerprint",
            },
        )
    assert resp.status_code == 200
    assert resp.json()["matched"] is True


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
