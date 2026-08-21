"""USSD adapter contract tests against the double."""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("USSD_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_root_menu() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/ussd/callback",
            data={
                "sessionId": "s1",
                "serviceCode": "*123#",
                "phoneNumber": "+254700000001",
                "text": "",
            },
        )
    assert resp.status_code == 200
    assert resp.text.startswith("CON")


@pytest.mark.asyncio
async def test_end_flow() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/ussd/callback",
            data={
                "sessionId": "s2",
                "serviceCode": "*123#",
                "phoneNumber": "+254700000001",
                "text": "1",
            },
        )
    assert resp.status_code == 200
    assert resp.text.startswith("END")


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
