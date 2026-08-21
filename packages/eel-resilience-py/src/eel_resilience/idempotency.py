from __future__ import annotations

import hashlib
import logging
from typing import Any, Protocol

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import Response

_logger = logging.getLogger(__name__)


class IdempotencyStore(Protocol):
    async def get(self, key: str) -> tuple[int, bytes, bytes] | None:
        """Return (status_code, body, request_body_hash) or None."""
        ...

    async def set(
        self,
        key: str,
        status_code: int,
        body: bytes,
        request_body_hash: bytes,
    ) -> None: ...


class InMemoryIdempotencyStore:
    """In-memory store for testing."""

    def __init__(self) -> None:
        self._store: dict[str, tuple[int, bytes, bytes]] = {}

    async def get(self, key: str) -> tuple[int, bytes, bytes] | None:
        return self._store.get(key)

    async def set(
        self,
        key: str,
        status_code: int,
        body: bytes,
        request_body_hash: bytes,
    ) -> None:
        self._store[key] = (status_code, body, request_body_hash)


class IdempotencyMiddleware(BaseHTTPMiddleware):
    """
    Reads Idempotency-Key header. For POST requests:
    - Checks store for existing result.
    - If found and body matches: replay cached response.
    - If found and body differs: return 422.
    - If not found: proceed, cache result.
    """

    def __init__(self, app: Any, store: IdempotencyStore) -> None:
        super().__init__(app)
        self._store = store

    async def dispatch(self, request: Request, call_next: Any) -> Response:
        if request.method != "POST":
            return await call_next(request)

        key = request.headers.get("Idempotency-Key")
        if not key:
            return await call_next(request)

        body = await request.body()
        body_hash = hashlib.sha256(body).digest()

        existing = await self._store.get(key)
        if existing is not None:
            cached_status, cached_body, cached_hash = existing
            if cached_hash != body_hash:
                _logger.warning("idempotency key=%s body_mismatch=true", key)
                return Response(
                    content=b'{"detail":"request body differs from original"}',
                    status_code=422,
                    media_type="application/json",
                )
            _logger.debug("idempotency key=%s replaying_cached=true", key)
            return Response(
                content=cached_body,
                status_code=cached_status,
            )

        response = await call_next(request)

        resp_body = b""
        async for chunk in response.body_iterator:
            if isinstance(chunk, bytes):
                resp_body += chunk
            else:
                resp_body += chunk.encode()

        await self._store.set(key, response.status_code, resp_body, body_hash)

        return Response(
            content=resp_body,
            status_code=response.status_code,
            headers=dict(response.headers),
            media_type=response.media_type,
        )
