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

import logging
from dataclasses import dataclass
from typing import Any

import httpx

_log = logging.getLogger(__name__)


@dataclass
class CheckResult:
    allowed: bool
    zookie: str


class SpiceDBClient:
    """SpiceDB client via HTTP JSON API (grpc-gateway)."""

    def __init__(
        self,
        base_url: str,
        token: str,
        client: httpx.AsyncClient | None = None,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._token = token
        self._client = client
        self._owns_client = client is None

    def _headers(self) -> dict[str, str]:
        return {
            "Authorization": f"Bearer {self._token}",
            "Content-Type": "application/json",
        }

    def _get_client(self) -> httpx.AsyncClient:
        if self._client is not None:
            return self._client
        return httpx.AsyncClient()

    async def check(
        self,
        resource_type: str,
        resource_id: str,
        permission: str,
        subject_type: str,
        subject_id: str,
        *,
        consistency: str = "minimize_latency",
    ) -> CheckResult:
        payload: dict[str, Any] = {
            "consistency": {consistency: True},
            "resource": {"objectType": resource_type, "objectId": resource_id},
            "permission": permission,
            "subject": {"object": {"objectType": subject_type, "objectId": subject_id}},
        }
        client = self._get_client()
        owned = self._client is None
        try:
            resp = await client.post(
                f"{self._base_url}/v1/permissions/check",
                json=payload,
                headers=self._headers(),
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await client.aclose()

        allowed = data.get("permissionship") == "PERMISSIONSHIP_HAS_PERMISSION"
        zookie: str = (
            data.get("checkedAt", {}).get("token", "")
            if isinstance(data.get("checkedAt"), dict)
            else ""
        )
        _log.debug(
            "SpiceDB check %s#%s@%s:%s -> %s",
            resource_type,
            permission,
            subject_type,
            subject_id,
            allowed,
        )
        return CheckResult(allowed=allowed, zookie=zookie)

    async def lookup_resources(
        self,
        resource_type: str,
        permission: str,
        subject_type: str,
        subject_id: str,
    ) -> list[str]:
        payload: dict[str, Any] = {
            "consistency": {"minimize_latency": True},
            "resourceObjectType": resource_type,
            "permission": permission,
            "subject": {"object": {"objectType": subject_type, "objectId": subject_id}},
        }
        client = self._get_client()
        owned = self._client is None
        resource_ids: list[str] = []
        try:
            resp = await client.post(
                f"{self._base_url}/v1/permissions/resources",
                json=payload,
                headers=self._headers(),
            )
            resp.raise_for_status()
            # Response is newline-delimited JSON stream
            for line in resp.text.splitlines():
                line = line.strip()
                if not line:
                    continue
                import json

                item: dict[str, Any] = json.loads(line)
                rid = item.get("result", {}).get("resourceObjectId")
                if rid:
                    resource_ids.append(str(rid))
        finally:
            if owned:
                await client.aclose()
        return resource_ids

    async def close(self) -> None:
        if self._owns_client and self._client is not None:
            await self._client.aclose()
