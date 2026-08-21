"""Pact consumer contract: eel services consuming the kbc-sftp adapter.

Provider verification runs against the kbc-sftp double (KBC_SFTP_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("KBC_SFTP_DOUBLE_URL", "http://localhost:8090")


async def test_kbc_bulletin_publish() -> None:
	"Upload hash bulletin via SFTP."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/kbc/bulletin/publish",
			json={"bulletinContent": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_kbc_bulletin_status() -> None:
	"Check last upload status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/kbc/bulletin/status",
			json={"bulletinContent": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
