"""Pact consumer contract: eel services consuming the social-platforms adapter.

Provider verification runs against the social-platforms double (SOCIAL_PLATFORMS_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("SOCIAL_PLATFORMS_DOUBLE_URL", "http://localhost:8090")


async def test_social_content_flag() -> None:
	"Flag social media content."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/social/content/flag",
			json={"contentUrl": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_social_content_status() -> None:
	"Check content moderation status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/social/content/status",
			json={"contentUrl": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
