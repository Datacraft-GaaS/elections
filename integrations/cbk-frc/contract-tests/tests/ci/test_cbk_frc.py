"""Pact consumer contract: eel services consuming the cbk-frc adapter.

Provider verification runs against the cbk-frc double (CBK_FRC_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("CBK_FRC_DOUBLE_URL", "http://localhost:8090")


async def test_cbk_sar_submit() -> None:
	"Submit suspicious activity report."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/cbk/sar/submit",
			json={"accountNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_cbk_sar_status() -> None:
	"Check SAR status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/cbk/sar/status",
			json={"accountNo": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
