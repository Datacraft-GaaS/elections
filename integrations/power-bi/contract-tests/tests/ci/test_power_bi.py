"""Pact consumer contract: eel services consuming the power-bi adapter.

Provider verification runs against the power-bi double (POWER_BI_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("POWER_BI_DOUBLE_URL", "http://localhost:8090")


async def test_powerbi_rows_push() -> None:
	"Push rows to Power BI dataset."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/powerbi/rows/push",
			json={"rowData": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_powerbi_dataset_refresh() -> None:
	"Trigger dataset refresh."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/powerbi/dataset/refresh",
			json={"rowData": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
