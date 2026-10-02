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

"""Contract tests for SMS adapter vs double."""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("SMS_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_send_single() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/version1/messaging",
            data={"username": "sandbox", "to": "+254700000001", "message": "Test"},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert data["SMSMessageData"]["Recipients"][0]["status"] == "Success"


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
