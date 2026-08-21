"""Pact consumer contract: eel services consuming the odpc-breach adapter.

Provider verification runs against the odpc-breach double (ODPC_BREACH_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("ODPC_BREACH_DOUBLE_URL", "http://localhost:8090")


async def test_odpc_breach_notify() -> None:
	"Submit 72h breach notification."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/odpc/breach/notify",
			json={"dataSubjects": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_odpc_breach_status() -> None:
	"Check notification status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/odpc/breach/status",
			json={"dataSubjects": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
