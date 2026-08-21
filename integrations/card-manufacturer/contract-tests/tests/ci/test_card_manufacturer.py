"""Pact consumer contract: eel services consuming the card-manufacturer adapter.

Provider verification runs against the card-manufacturer double (CARD_MANUFACTURER_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("CARD_MANUFACTURER_DOUBLE_URL", "http://localhost:8090")


async def test_card_order_submit() -> None:
	"Submit card production order."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/card/order/submit",
			json={"voterNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_card_order_status() -> None:
	"Check order status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/card/order/status",
			json={"voterNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_card_dispatch_confirm() -> None:
	"Confirm card dispatch."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/card/dispatch/confirm",
			json={"voterNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
