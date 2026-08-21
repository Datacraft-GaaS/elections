"""Pact consumer contract: eel services consuming the kra-eclearance adapter.

Provider verification runs against the kra-eclearance double (KRA_ECLEARANCE_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("KRA_ECLEARANCE_DOUBLE_URL", "http://localhost:8090")


async def test_kra_eclearance_entry_submit() -> None:
	"Submit customs entry."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/kra/eclearance/entry/submit",
			json={"importerPin": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_kra_eclearance_entry_status() -> None:
	"Check entry status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/kra/eclearance/entry/status",
			json={"importerPin": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
