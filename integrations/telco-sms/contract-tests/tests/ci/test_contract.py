"""Contract tests for SMS adapter vs double."""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("SMS_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_send_single() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/version1/messaging",
            data={"username": "sandbox", "to": "+254700000001", "message": "Test"},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert data["SMSMessageData"]["Recipients"][0]["status"] == "Success"


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
