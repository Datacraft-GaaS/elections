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
from typing import Any

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse, Response
from starlette.types import ASGIApp

from eel_auth.jwks import AuthError, JwksCache, verify_token
from eel_auth.subject import Subject, SubjectExtractor

_log = logging.getLogger(__name__)


class AuthMiddleware(BaseHTTPMiddleware):
    """Validates Bearer JWT and sets request.state.subject."""

    def __init__(
        self,
        app: ASGIApp,
        jwks_cache: JwksCache,
        extractor: SubjectExtractor,
        *,
        exclude_paths: list[str] | None = None,
        audience: str | None = None,
        issuer: str | None = None,
    ) -> None:
        super().__init__(app)
        self._jwks_cache = jwks_cache
        self._extractor = extractor
        self._exclude_paths = set(exclude_paths or [])
        self._audience = audience
        self._issuer = issuer

    async def dispatch(self, request: Request, call_next: Any) -> Response:
        if request.url.path in self._exclude_paths:
            return await call_next(request)

        auth_header = request.headers.get("Authorization", "")
        if not auth_header.startswith("Bearer "):
            return JSONResponse({"detail": "Missing Bearer token"}, status_code=401)

        token = auth_header[len("Bearer ") :]
        try:
            claims = await verify_token(
                token,
                self._jwks_cache,
                audience=self._audience,
                issuer=self._issuer,
            )
        except AuthError as exc:
            _log.debug("Auth failed: %s", exc)
            return JSONResponse({"detail": str(exc)}, status_code=401)

        request.state.subject = self._extractor.extract(claims)
        return await call_next(request)


def require_permission(permission: str, resource_type: str = "") -> Any:
    """FastAPI dependency that checks SpiceDB permission.

    Usage:
        @app.get("/resource", dependencies=[Depends(require_permission("read", "document"))])
    """
    from fastapi import Depends, HTTPException, Request

    from eel_auth.spicedb import SpiceDBClient

    async def _check(request: Request) -> Subject:
        subject: Subject | None = getattr(request.state, "subject", None)
        if subject is None:
            raise HTTPException(status_code=401, detail="Not authenticated")

        spicedb: SpiceDBClient | None = getattr(request.app.state, "spicedb", None)
        if spicedb is None:
            raise HTTPException(status_code=500, detail="SpiceDB client not configured")

        resource_id = request.path_params.get("id", "*")
        result = await spicedb.check(
            resource_type=resource_type,
            resource_id=resource_id,
            permission=permission,
            subject_type="user",
            subject_id=subject.sub,
        )
        if not result.allowed:
            raise HTTPException(status_code=403, detail="Permission denied")
        return subject

    return Depends(_check)
