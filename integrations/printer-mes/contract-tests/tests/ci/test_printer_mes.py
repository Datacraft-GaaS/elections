"""Pact consumer contract: eel services consuming the printer-mes adapter.

Provider verification runs against the printer-mes double (PRINTER_MES_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("PRINTER_MES_DOUBLE_URL", "http://localhost:8090")


async def test_printer_order_submit() -> None:
	"Submit print order."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/printer/order/submit",
			json={"printOrderRef": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_printer_order_status() -> None:
	"Check print order status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/printer/order/status",
			json={"printOrderRef": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
