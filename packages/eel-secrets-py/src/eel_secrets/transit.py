import base64
import logging
from typing import Any

from eel_secrets.client import OpenBaoClient

_log = logging.getLogger(__name__)


class Transit:
    """OpenBao Transit secrets engine wrapper."""

    def __init__(self, client: OpenBaoClient, mount_path: str = "transit") -> None:
        self._client = client
        self._mount = mount_path.strip("/")

    async def encrypt(self, key_name: str, plaintext: bytes) -> str:
        """Encrypt plaintext bytes. Returns ciphertext string."""
        await self._client._ensure_authed()
        b64 = base64.b64encode(plaintext).decode()
        http = self._client._get_client()
        owned = self._client._client is None
        try:
            resp = await http.post(
                f"{self._client._base_url}/v1/{self._mount}/encrypt/{key_name}",
                json={"plaintext": b64},
                headers=self._client._headers(),
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await http.aclose()
        ciphertext: str = data["data"]["ciphertext"]
        return ciphertext

    async def decrypt(self, key_name: str, ciphertext: str) -> bytes:
        """Decrypt ciphertext string. Returns plaintext bytes."""
        await self._client._ensure_authed()
        http = self._client._get_client()
        owned = self._client._client is None
        try:
            resp = await http.post(
                f"{self._client._base_url}/v1/{self._mount}/decrypt/{key_name}",
                json={"ciphertext": ciphertext},
                headers=self._client._headers(),
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await http.aclose()
        b64: str = data["data"]["plaintext"]
        return base64.b64decode(b64)

    async def rotate_key(self, key_name: str) -> None:
        """Rotate a Transit key."""
        await self._client._ensure_authed()
        http = self._client._get_client()
        owned = self._client._client is None
        try:
            resp = await http.post(
                f"{self._client._base_url}/v1/{self._mount}/keys/{key_name}/rotate",
                headers=self._client._headers(),
            )
            resp.raise_for_status()
        finally:
            if owned:
                await http.aclose()
        _log.info("Transit: rotated key %s", key_name)
