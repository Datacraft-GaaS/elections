"""Pact consumer contract: eel services consuming the kebs-warranty adapter.

Provider verification runs against the kebs-warranty double (KEBS_WARRANTY_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("KEBS_WARRANTY_DOUBLE_URL", "http://localhost:8090")


async def test_kebs_certification_verify() -> None:
	"Verify product certification."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/kebs/certification/verify",
			json={"serialNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_kebs_warranty_status() -> None:
	"Check warranty status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/kebs/warranty/status",
			json={"serialNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
