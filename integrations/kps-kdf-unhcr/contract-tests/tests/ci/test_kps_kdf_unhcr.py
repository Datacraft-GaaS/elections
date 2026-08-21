"""Pact consumer contract: eel services consuming the kps-kdf-unhcr adapter.

Provider verification runs against the kps-kdf-unhcr double (KPS_KDF_UNHCR_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("KPS_KDF_UNHCR_DOUBLE_URL", "http://localhost:8090")


async def test_displaced_register() -> None:
	"Register displaced voter."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/displaced/register",
			json={"refugeeId": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_displaced_status() -> None:
	"Check displaced voter status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/displaced/status",
			json={"refugeeId": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
