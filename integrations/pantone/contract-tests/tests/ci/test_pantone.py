"""Pact consumer contract: eel services consuming the pantone adapter.

Provider verification runs against the pantone double (PANTONE_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("PANTONE_DOUBLE_URL", "http://localhost:8090")


async def test_pantone_colour_lookup() -> None:
	"Lookup Pantone colour values."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/pantone/colour/lookup",
			json={"colourCode": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_pantone_colour_match() -> None:
	"Find nearest Pantone match."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/pantone/colour/match",
			json={"colourCode": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
