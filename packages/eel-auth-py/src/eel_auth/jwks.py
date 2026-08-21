import asyncio
import logging
import time
from typing import Any

import httpx
from jose import jwt, JWTError

_log = logging.getLogger(__name__)


class AuthError(Exception):
    """Raised on JWT verification failure."""

    pass


class JwksCache:
    """Caches JWKS from a Keycloak realm, refreshing on kid-miss."""

    def __init__(
        self,
        jwks_url: str,
        client: httpx.AsyncClient | None = None,
        ttl_s: int = 300,
    ) -> None:
        self._jwks_url = jwks_url
        self._client = client
        self._owns_client = client is None
        self._ttl_s = ttl_s
        self._keys: dict[str, dict[str, Any]] = {}
        self._fetched_at: float = 0.0
        self._lock = asyncio.Lock()

    def _log_fetch(self, url: str) -> None:
        _log.debug("JwksCache: fetching JWKS from %s", url)

    async def get_key(self, kid: str) -> dict[str, Any]:
        """Return JWK dict for kid; fetches from remote if miss or TTL expired."""
        now = time.monotonic()
        if kid not in self._keys or (now - self._fetched_at) > self._ttl_s:
            async with self._lock:
                # Re-check after acquiring lock
                now = time.monotonic()
                if kid not in self._keys or (now - self._fetched_at) > self._ttl_s:
                    await self._fetch()
        if kid not in self._keys:
            raise AuthError(f"Unknown kid: {kid!r}")
        return self._keys[kid]

    async def _fetch(self) -> None:
        self._log_fetch(self._jwks_url)
        client = self._client or httpx.AsyncClient()
        owned = self._client is None
        try:
            resp = await client.get(self._jwks_url)
            resp.raise_for_status()
            data = resp.json()
        except httpx.HTTPError as exc:
            raise AuthError(f"JWKS fetch failed: {exc}") from exc
        finally:
            if owned:
                await client.aclose()
        keys: dict[str, dict[str, Any]] = {}
        for jwk in data.get("keys", []):
            kid = jwk.get("kid")
            if kid:
                keys[kid] = jwk
        self._keys = keys
        self._fetched_at = time.monotonic()
        _log.debug("JwksCache: loaded %d keys", len(keys))

    async def close(self) -> None:
        if self._owns_client and self._client is not None:
            await self._client.aclose()


async def verify_token(
    token: str,
    jwks_cache: JwksCache,
    *,
    audience: str | None = None,
    issuer: str | None = None,
) -> dict[str, Any]:
    """Decode and verify a JWT. Returns claims dict. Raises AuthError on failure."""
    try:
        unverified_header = jwt.get_unverified_header(token)
    except JWTError as exc:
        raise AuthError(f"Invalid token header: {exc}") from exc

    kid = unverified_header.get("kid")
    if not kid:
        raise AuthError("Token header missing 'kid'")

    jwk_key = await jwks_cache.get_key(kid)

    options: dict[str, Any] = {}
    if audience is None:
        options["verify_aud"] = False

    try:
        claims: dict[str, Any] = jwt.decode(
            token,
            jwk_key,
            algorithms=["RS256", "ES256"],
            audience=audience,
            issuer=issuer,
            options=options,
        )
    except JWTError as exc:
        raise AuthError(f"Token verification failed: {exc}") from exc

    return claims
