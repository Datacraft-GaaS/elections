import logging
import time
from typing import Any

import httpx

_log = logging.getLogger(__name__)


class OpenBaoClient:
    """Async OpenBao client. Supports AppRole auth and KV v2."""

    def __init__(
        self,
        base_url: str,
        *,
        role_id: str | None = None,
        secret_id: str | None = None,
        token: str | None = None,
        client: httpx.AsyncClient | None = None,
        token_renew_buffer_s: int = 30,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._role_id = role_id
        self._secret_id = secret_id
        self._token: str | None = token
        self._client = client
        self._owns_client = client is None
        self._token_renew_buffer_s = token_renew_buffer_s
        self._token_expires_at: float = 0.0

    def _get_client(self) -> httpx.AsyncClient:
        if self._client is not None:
            return self._client
        return httpx.AsyncClient()

    def _headers(self) -> dict[str, str]:
        assert self._token is not None, "No token available"
        return {"X-Vault-Token": self._token}

    async def auth_approle(self, role_id: str, secret_id: str) -> str:
        """Authenticate with AppRole and return the token."""
        client = self._get_client()
        owned = self._client is None
        try:
            resp = await client.post(
                f"{self._base_url}/v1/auth/approle/login",
                json={"role_id": role_id, "secret_id": secret_id},
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await client.aclose()

        auth = data["auth"]
        token: str = str(auth["client_token"])
        self._token = token
        lease_duration: int = auth.get("lease_duration", 0)
        self._token_expires_at = time.monotonic() + lease_duration
        _log.debug(
            "OpenBaoClient: authenticated via AppRole, lease=%ds", lease_duration
        )
        return token

    async def _ensure_authed(self) -> None:
        """Lazy auth with renewal when token is near expiry."""
        now = time.monotonic()
        needs_auth = self._token is None or (
            self._token_expires_at > 0
            and (self._token_expires_at - now) < self._token_renew_buffer_s
        )
        if needs_auth:
            if self._role_id is None or self._secret_id is None:
                raise RuntimeError("No token and no AppRole credentials configured")
            await self.auth_approle(self._role_id, self._secret_id)

    async def close(self) -> None:
        if self._owns_client and self._client is not None:
            await self._client.aclose()

    async def read_secret(self, path: str) -> dict[str, Any]:
        """Read a KV v2 secret. Returns the data dict."""
        await self._ensure_authed()
        # KV v2: insert /data/ after the mount
        parts = path.split("/", 1)
        kv_path = f"{parts[0]}/data/{parts[1]}" if len(parts) == 2 else f"{path}/data"
        client = self._get_client()
        owned = self._client is None
        try:
            resp = await client.get(
                f"{self._base_url}/v1/{kv_path}",
                headers=self._headers(),
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await client.aclose()
        return data.get("data", {}).get("data", {})

    async def write_secret(self, path: str, data: dict[str, Any]) -> None:
        """Write a KV v2 secret."""
        await self._ensure_authed()
        parts = path.split("/", 1)
        kv_path = f"{parts[0]}/data/{parts[1]}" if len(parts) == 2 else f"{path}/data"
        client = self._get_client()
        owned = self._client is None
        try:
            resp = await client.post(
                f"{self._base_url}/v1/{kv_path}",
                json={"data": data},
                headers=self._headers(),
            )
            resp.raise_for_status()
        finally:
            if owned:
                await client.aclose()
