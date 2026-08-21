"""Pact consumer contract: eel services consuming the ecourt adapter.

Provider verification runs against the ecourt double (ECOURT_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("ECOURT_DOUBLE_URL", "http://localhost:8090")


async def test_ecourt_petition_file() -> None:
	"File election petition."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/ecourt/petition/file",
			json={"petitionerName": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_ecourt_document_upload() -> None:
	"Upload supporting document."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/ecourt/document/upload",
			json={"petitionerName": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_ecourt_case_status() -> None:
	"Check case status."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/ecourt/case/status",
			json={"petitionerName": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
