"""Pact consumer contract: eel services consuming the ppdt adapter.

Provider verification runs against the ppdt double (PPDT_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("PPDT_DOUBLE_URL", "http://localhost:8090")


async def test_ppdt_party_lookup() -> None:
	"Lookup party details."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/ppdt/party/lookup",
			json={"partyName": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_ppdt_coalition_register() -> None:
	"Register coalition."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/ppdt/coalition/register",
			json={"partyName": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_ppdt_compliance_check() -> None:
	"Check compliance status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/ppdt/compliance/check",
			json={"partyName": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
