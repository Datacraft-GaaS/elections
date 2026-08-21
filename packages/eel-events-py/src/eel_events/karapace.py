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
