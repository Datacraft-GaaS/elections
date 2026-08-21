"""Pact consumer contract: eel services consuming the mpesa-daraja adapter.

Provider verification runs against the mpesa-daraja double (MPESA_DARAJA_DOUBLE_URL env var).
"""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("MPESA_DARAJA_DOUBLE_URL", "http://localhost:8090")


async def test_mpesa_c2b_register() -> None:
	"Register C2B URL."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/mpesa/c2b/register",
			json={"phoneNumber": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_mpesa_c2b_callback() -> None:
	"Receive payment notification."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/mpesa/c2b/callback",
			json={"phoneNumber": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_mpesa_b2c_pay() -> None:
	"Initiate B2C payment."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.post(
			"/mpesa/b2c/pay",
			json={"phoneNumber": "test-value", "request_id": "00000000-0000-0000-0000-000000000001"},
		)
	assert resp.status_code == 200
	data = resp.json()
	assert data["ok"] is True


async def test_health() -> None:
	"Health endpoint."
	async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
		resp = await client.get("/health")
	assert resp.status_code == 200
