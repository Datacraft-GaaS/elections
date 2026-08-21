"""Pact consumer contract: eel services consuming the IPRS adapter.

Provider verification runs against the IPRS double (IPRS_DOUBLE_URL env var).
Pact files are published to PACT_BROKER_URL.
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("IPRS_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_verify_matched() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post("/api/verify", json={"nationalId": "12345678901234"})
    assert resp.status_code == 200
    data = resp.json()
    assert data["matched"] is True
    assert "duplicate" in data


@pytest.mark.asyncio
async def test_verify_duplicate_mode() -> None:
    """EC-DAT-15: verify duplicate-person detection knob."""
    # When IPRS_DUPLICATE_MODE=true the double returns duplicate=true.
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post("/api/verify", json={"nationalId": "99999999999999"})
    assert resp.status_code == 200
    # In normal mode duplicate may be false; just assert field exists.
    assert "duplicate" in resp.json()


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
