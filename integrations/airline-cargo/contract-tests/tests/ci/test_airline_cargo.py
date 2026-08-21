"""Pact consumer contract: eel services consuming the airline-cargo adapter.

Provider verification runs against the airline-cargo double (AIRLINE_CARGO_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("AIRLINE_CARGO_DOUBLE_URL", "http://localhost:8090")


async def test_airline_manifest_submit() -> None:
	"Submit cargo manifest."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/airline/manifest/submit",
			json={"manifestRef": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_airline_manifest_status() -> None:
	"Check manifest clearance status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/airline/manifest/status",
			json={"manifestRef": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
