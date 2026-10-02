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

"""eel_web middlewares: auth, audit, PII scrubbing."""

from __future__ import annotations

import json
import logging
import re
from typing import Any

from django.http import HttpRequest, HttpResponse, JsonResponse
from django.utils.deprecation import MiddlewareMixin

_log = logging.getLogger(__name__)

# Paths that skip JWT validation
EXEMPT_PATHS: frozenset[str] = frozenset(
    [
        "/health/live",
        "/health/ready",
        "/favicon.ico",
    ]
)

_PII_PATTERNS: list[re.Pattern[str]] = [
    re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),  # SSN
    re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"),  # email
]


class EelAuthMiddleware(MiddlewareMixin):
    """Validates Bearer JWT; populates request.eel_subject."""

    def process_request(self, request: HttpRequest) -> HttpResponse | None:
        if request.path in EXEMPT_PATHS:
            return None

        auth_header: str = request.META.get("HTTP_AUTHORIZATION", "")
        if not auth_header.startswith("Bearer "):
            # No token; allow through (protected views enforce auth themselves)
            request.eel_subject = None  # type: ignore[attr-defined]
            return None

        token = auth_header.removeprefix("Bearer ").strip()
        subject = self._verify_token(token)
        if subject is None:
            return JsonResponse({"detail": "Invalid or expired token."}, status=401)

        request.eel_subject = subject  # type: ignore[attr-defined]
        return None

    def _verify_token(self, token: str) -> str | None:
        """Stub: verify JWT and return subject claim. Replace with JWKS validation."""
        # Real impl: fetch JWKS from Keycloak, verify signature, return sub claim
        try:
            import base64 as _b64

            parts = token.split(".")
            if len(parts) != 3:
                return None
            payload_bytes = parts[1] + "=" * (-len(parts[1]) % 4)
            payload: dict[str, Any] = json.loads(_b64.urlsafe_b64decode(payload_bytes))
            return payload.get("sub")
        except Exception:
            return None


class EelAuditMiddleware(MiddlewareMixin):
    """Logs requests with subject info."""

    def process_response(
        self, request: HttpRequest, response: HttpResponse
    ) -> HttpResponse:
        subject = getattr(request, "eel_subject", None)
        _log.info(
            "audit: %s %s status=%s subject=%s",
            request.method,
            request.path,
            response.status_code,
            subject or "anonymous",
        )
        return response


class EelPiiScrubberMiddleware(MiddlewareMixin):
    """Strips PII from exception reports and log messages."""

    def process_exception(self, request: HttpRequest, exception: Exception) -> None:
        msg = str(exception)
        for pattern in _PII_PATTERNS:
            msg = pattern.sub("[REDACTED]", msg)
        _log.error("exception (scrubbed): %s path=%s", msg, request.path)
        return None
