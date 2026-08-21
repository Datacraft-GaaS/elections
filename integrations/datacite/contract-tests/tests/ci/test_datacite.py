"""Pact consumer contract: eel services consuming the datacite adapter.

Provider verification runs against the datacite double (DATACITE_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("DATACITE_DOUBLE_URL", "http://localhost:8090")


async def test_datacite_doi_mint() -> None:
	"Mint a new DOI."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/datacite/doi/mint",
			json={"title": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_datacite_doi_resolve() -> None:
	"Resolve DOI metadata."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/datacite/doi/resolve",
			json={"title": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
