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

"""Karapace schema registry client."""

from __future__ import annotations

import logging

import httpx

logger = logging.getLogger(__name__)


class KarapaceClient:
    """Async client for the Karapace schema registry."""

    def __init__(self, base_url: str, client: httpx.AsyncClient | None = None) -> None:
        self._base_url = base_url.rstrip("/")
        self._owned_client = client is None
        self._client = client or httpx.AsyncClient(base_url=self._base_url)

    async def get_schema(self, subject: str, version: str = "latest") -> dict:
        """Fetch schema for a subject at a given version."""
        resp = await self._client.get(f"/subjects/{subject}/versions/{version}")
        resp.raise_for_status()
        return resp.json()

    async def register_schema(self, subject: str, schema: dict) -> int:
        """Register a schema for a subject. Returns the schema id."""
        resp = await self._client.post(
            f"/subjects/{subject}/versions",
            json={"schema": schema},
        )
        resp.raise_for_status()
        return resp.json()["id"]

    async def check_compatibility(self, subject: str, schema: dict) -> bool:
        """Check if schema is compatible with the latest registered schema."""
        resp = await self._client.post(
            f"/compatibility/subjects/{subject}/versions/latest",
            json={"schema": schema},
        )
        resp.raise_for_status()
        return resp.json().get("is_compatible", False)

    async def close(self) -> None:
        """Close the underlying HTTP client if it was created internally."""
        if self._owned_client:
            await self._client.aclose()
