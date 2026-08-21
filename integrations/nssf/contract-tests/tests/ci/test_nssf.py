"""Pact consumer contract: eel services consuming the nssf adapter.

Provider verification runs against the nssf double (NSSF_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("NSSF_DOUBLE_URL", "http://localhost:8090")


async def test_nssf_member_lookup() -> None:
	"Lookup NSSF member."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/nssf/member/lookup",
			json={"nationalId": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_nssf_contribution_remit() -> None:
	"Submit contribution."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/nssf/contribution/remit",
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
