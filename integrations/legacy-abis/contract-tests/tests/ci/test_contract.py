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

"""Legacy ABIS contract tests against the double."""

from __future__ import annotations

import os

import httpx
import pytest

DOUBLE_URL = os.environ.get("ABIS_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_match_1n() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/abis/1n",
            json={"probe": "AQID", "modality": "fingerprint", "threshold": 0.8},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert data["matched"] is True
    assert "score" in data


@pytest.mark.asyncio
async def test_match_11() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.post(
            "/abis/11",
            json={
                "probe": "AQID",
                "referenceId": "voter-001",
                "modality": "fingerprint",
            },
        )
    assert resp.status_code == 200
    assert resp.json()["matched"] is True


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
