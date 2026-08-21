"""Pact consumer contract: eel services consuming the eacc adapter.

Provider verification runs against the eacc double (EACC_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("EACC_DOUBLE_URL", "http://localhost:8090")


async def test_eacc_clearance_check() -> None:
	"Check EACC integrity clearance."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/eacc/clearance/check",
			json={"nationalId": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
