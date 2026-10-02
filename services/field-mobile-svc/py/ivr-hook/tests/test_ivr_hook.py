# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Tests for ivr-hook service."""

from __future__ import annotations

import pytest
from httpx import ASGITransport, AsyncClient

from ivr_hook.main import app
from ivr_hook.tts.builder import build_menu, build_ssml


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


async def test_voice_webhook_first_call(client: AsyncClient) -> None:
    resp = await client.post(
        "/ivr/voice",
        data={"sessionId": "sess001", "phoneNumber": "+254700000001"},
    )
    assert resp.status_code == 200
    body = resp.text
    assert "<speak" in body
    assert "Press 1" in body


async def test_voice_webhook_dtmf_verify(client: AsyncClient) -> None:
    resp = await client.post(
        "/ivr/voice",
        data={
            "sessionId": "sess002",
            "phoneNumber": "+254700000002",
            "dtmfDigits": "1",
        },
    )
    assert resp.status_code == 200
    assert "verification" in resp.text.lower()


async def test_build_ssml_escapes_ampersand() -> None:
    result = build_ssml("Q&A session", lang="en")
    assert "&amp;" in result
    assert "<speak" in result


async def test_build_menu_returns_ssml() -> None:
    result = build_menu([("1", "verify"), ("2", "lookup")])
    assert "Press 1 for verify" in result
    assert "<speak" in result
