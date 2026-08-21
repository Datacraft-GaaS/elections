"""Pact consumer contract: eel services consuming the gov-printer adapter.

Provider verification runs against the gov-printer double (GOV_PRINTER_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("GOV_PRINTER_DOUBLE_URL", "http://localhost:8090")


async def test_gov_printer_gazette_submit() -> None:
	"Submit gazette document."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/gov-printer/gazette/submit",
			json={"documentRef": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_gov_printer_gazette_status() -> None:
	"Check submission status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/gov-printer/gazette/status",
			json={"documentRef": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
