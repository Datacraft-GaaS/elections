import logging
from dataclasses import dataclass
from typing import Any

from eel_secrets.client import OpenBaoClient

_log = logging.getLogger(__name__)


@dataclass
class PgCredential:
    username: str
    password: str
    lease_id: str
    lease_duration_s: int


class PgDynamicCred:
    """Fetches and renews dynamic PostgreSQL credentials from OpenBao."""

    MAX_LEASE_SECONDS = 4 * 3600  # 14400

    def __init__(
        self,
        client: OpenBaoClient,
        db_role: str,
        mount_path: str = "database",
    ) -> None:
        self._client = client
        self._db_role = db_role
        self._mount = mount_path.strip("/")

    async def get(self) -> PgCredential:
        """Fetch a fresh dynamic PG credential."""
        await self._client._ensure_authed()
        http = self._client._get_client()
        owned = self._client._client is None
        try:
            resp = await http.get(
                f"{self._client._base_url}/v1/{self._mount}/creds/{self._db_role}",
                headers=self._client._headers(),
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await http.aclose()

        duration = min(int(data.get("lease_duration", 0)), self.MAX_LEASE_SECONDS)
        return PgCredential(
            username=data["data"]["username"],
            password=data["data"]["password"],
            lease_id=data.get("lease_id", ""),
            lease_duration_s=duration,
        )

    async def renew(self, credential: PgCredential) -> PgCredential:
        """Renew a lease and return an updated credential."""
        await self._client._ensure_authed()
        http = self._client._get_client()
        owned = self._client._client is None
        try:
            resp = await http.put(
                f"{self._client._base_url}/v1/sys/leases/renew",
                json={"lease_id": credential.lease_id},
                headers=self._client._headers(),
            )
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
        finally:
            if owned:
                await http.aclose()

        duration = min(
            int(data.get("lease_duration", credential.lease_duration_s)),
            self.MAX_LEASE_SECONDS,
        )
        return PgCredential(
            username=credential.username,
            password=credential.password,
            lease_id=data.get("lease_id", credential.lease_id),
            lease_duration_s=duration,
        )

    async def revoke(self, credential: PgCredential) -> None:
        """Revoke a lease immediately."""
        await self._client._ensure_authed()
        http = self._client._get_client()
        owned = self._client._client is None
        try:
            resp = await http.put(
                f"{self._client._base_url}/v1/sys/leases/revoke",
                json={"lease_id": credential.lease_id},
                headers=self._client._headers(),
            )
            resp.raise_for_status()
        finally:
            if owned:
                await http.aclose()
        _log.info("PgDynamicCred: revoked lease %s", credential.lease_id)

    def connection_string(
        self,
        credential: PgCredential,
        host: str,
        port: int,
        dbname: str,
    ) -> str:
        return f"postgresql://{credential.username}:{credential.password}@{host}:{port}/{dbname}"
