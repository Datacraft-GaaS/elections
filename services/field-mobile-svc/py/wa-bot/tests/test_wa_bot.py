"""Tests for wa-bot webhook service."""

from __future__ import annotations

import pytest
from httpx import ASGITransport, AsyncClient

from wa_bot.main import app


@pytest.fixture
async def client():
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as c:
        yield c


async def test_health(client: AsyncClient) -> None:
    resp = await client.get("/healthz")
    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"


async def test_webhook_verify_valid_token(client: AsyncClient) -> None:
    resp = await client.get(
        "/webhook/whatsapp",
        params={
            "hub_mode": "subscribe",
            "hub_challenge": "12345",
            "hub_verify_token": "eel-verify-token",
        },
    )
    assert resp.status_code == 200
    assert resp.json() == 12345


async def test_webhook_verify_invalid_token(client: AsyncClient) -> None:
    resp = await client.get(
        "/webhook/whatsapp",
        params={
            "hub_mode": "subscribe",
            "hub_challenge": "99999",
            "hub_verify_token": "wrong-token",
        },
    )
    assert resp.status_code == 403


async def test_webhook_post_receives_message(client: AsyncClient) -> None:
    payload = {
        "object": "whatsapp_business_account",
        "entry": [
            {
                "id": "ENTRY_ID",
                "changes": [
                    {
                        "value": {
                            "messages": [
                                {
                                    "id": "MSG_ID",
                                    "from": "254700000001",
                                    "type": "text",
                                    "text": {"body": "verify my registration"},
                                }
                            ]
                        },
                        "field": "messages",
                    }
                ],
            }
        ],
    }
    resp = await client.post("/webhook/whatsapp", json=payload)
    assert resp.status_code == 200
    assert resp.json()["status"] == "received"
