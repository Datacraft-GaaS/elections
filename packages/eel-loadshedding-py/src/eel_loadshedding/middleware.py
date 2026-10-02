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

from __future__ import annotations

import asyncio
import logging
from enum import Enum
from typing import Any, Callable

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse, Response

_logger = logging.getLogger(__name__)


class Priority(Enum):
    CRITICAL = 0  # election officials, system health
    HIGH = 1  # authenticated users
    NORMAL = 2  # regular requests
    LOW = 3  # background/batch


def shed_response(retry_after_s: int = 5) -> JSONResponse:
    """Return RFC 9457 problem+json 503 response."""
    body = {
        "type": "urn:eel:load-shedding",
        "title": "Service Unavailable",
        "status": 503,
        "detail": "Server is overloaded. Please retry after the indicated interval.",
        "retry_after": retry_after_s,
    }
    return JSONResponse(
        content=body,
        status_code=503,
        headers={
            "Retry-After": str(retry_after_s),
            "Content-Type": "application/problem+json",
        },
        media_type="application/problem+json",
    )


def default_priority_extractor(request: Request) -> Priority:
    """Extract priority from path or X-Priority header."""
    path = request.url.path
    if path == "/health" or path.startswith("/health/"):
        return Priority.CRITICAL

    header = request.headers.get("X-Priority", "").lower()
    mapping: dict[str, Priority] = {
        "critical": Priority.CRITICAL,
        "high": Priority.HIGH,
        "normal": Priority.NORMAL,
        "low": Priority.LOW,
    }
    return mapping.get(header, Priority.NORMAL)


class LoadSheddingMiddleware(BaseHTTPMiddleware):
    """
    CoDel-inspired load shedder. Tracks active request count.
    When overloaded, sheds LOW priority first, then NORMAL, etc.

    Returns 503 with:
    - Retry-After header
    - Content-Type: application/problem+json
    - RFC 9457 problem+json body
    """

    def __init__(
        self,
        app: Any,
        *,
        max_queue_depth: int = 100,
        target_delay_ms: float = 5.0,
        interval_ms: float = 100.0,
        priority_extractor: Callable[[Request], Priority] | None = None,
        retry_after_s: int = 5,
    ) -> None:
        super().__init__(app)
        self.max_queue_depth = max_queue_depth
        self.target_delay_ms = target_delay_ms
        self.interval_ms = interval_ms
        self.priority_extractor = priority_extractor or default_priority_extractor
        self.retry_after_s = retry_after_s

        self._active = 0
        self._lock = asyncio.Lock()

    def _should_shed(self, priority: Priority) -> bool:
        """Determine if request should be shed based on queue depth and priority."""
        depth = self._active
        if depth < self.max_queue_depth:
            return False
        # When at capacity, shed LOW; at 2x capacity shed NORMAL too, etc.
        shed_threshold: dict[Priority, float] = {
            Priority.LOW: 1.0,
            Priority.NORMAL: 1.5,
            Priority.HIGH: 2.0,
            Priority.CRITICAL: float("inf"),
        }
        ratio = depth / self.max_queue_depth
        return ratio >= shed_threshold.get(priority, 1.0)

    async def dispatch(self, request: Request, call_next: Any) -> Response:
        priority = self.priority_extractor(request)

        async with self._lock:
            if self._should_shed(priority):
                _logger.warning(
                    "load_shedding priority=%s active=%d max=%d",
                    priority.name,
                    self._active,
                    self.max_queue_depth,
                )
                return shed_response(self.retry_after_s)
            self._active += 1

        try:
            return await call_next(request)
        finally:
            async with self._lock:
                self._active -= 1
