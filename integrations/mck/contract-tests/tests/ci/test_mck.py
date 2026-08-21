"""Pact consumer contract: eel services consuming the mck adapter.

Provider verification runs against the mck double (MCK_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("MCK_DOUBLE_URL", "http://localhost:8090")


async def test_mck_journalist_status() -> None:
	"Check journalist accreditation."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/mck/journalist/status",
			json={"journalistId": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_mck_media_lookup() -> None:
	"Lookup media house."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/mck/media/lookup",
			json={"journalistId": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
