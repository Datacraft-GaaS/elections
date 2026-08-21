"""Pact consumer contract: eel services consuming the satellite adapter (Rust).

Provider verification runs against the satellite double (SATELLITE_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("SATELLITE_DOUBLE_URL", "http://localhost:8090")


async def test_satellite_status() -> None:
	"Get satellite link status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/satellite/status",
			json={"request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_satellite_failover_activate() -> None:
	"Activate fallback bearer."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/satellite/failover/activate",
			json={"request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
