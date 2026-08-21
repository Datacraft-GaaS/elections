from __future__ import annotations

import asyncio
import logging
import time
from enum import Enum
from typing import Any

_logger = logging.getLogger(__name__)


class CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class CircuitBreakerOpen(Exception):
    def __init__(self, name: str) -> None:
        super().__init__(f"Circuit breaker '{name}' is OPEN")
        self.name = name


class CircuitBreaker:
    def __init__(
        self,
        name: str,
        failure_threshold: int = 5,
        recovery_timeout_s: float = 60.0,
        half_open_max_calls: int = 1,
    ) -> None:
        self.name = name
        self.failure_threshold = failure_threshold
        self.recovery_timeout_s = recovery_timeout_s
        self.half_open_max_calls = half_open_max_calls

        self._state = CircuitState.CLOSED
        self._failure_count = 0
        self._last_failure_time: float | None = None
        self._half_open_calls = 0
        self._lock = asyncio.Lock()

    @property
    def state(self) -> CircuitState:
        if self._state == CircuitState.OPEN:
            assert self._last_failure_time is not None
            elapsed = time.monotonic() - self._last_failure_time
            if elapsed >= self.recovery_timeout_s:
                self._state = CircuitState.HALF_OPEN
                self._half_open_calls = 0
                _logger.info("circuit_breaker name=%s transition=HALF_OPEN", self.name)
        return self._state

    async def call(self, coro: Any) -> Any:
        async with self._lock:
            current = self.state
            if current == CircuitState.OPEN:
                raise CircuitBreakerOpen(self.name)
            if current == CircuitState.HALF_OPEN:
                if self._half_open_calls >= self.half_open_max_calls:
                    raise CircuitBreakerOpen(self.name)
                self._half_open_calls += 1

        try:
            result = await coro
            self._on_success()
            return result
        except Exception:
            self._on_failure()
            raise

    def _on_success(self) -> None:
        self._failure_count = 0
        self._last_failure_time = None
        if self._state != CircuitState.CLOSED:
            _logger.info("circuit_breaker name=%s transition=CLOSED", self.name)
        self._state = CircuitState.CLOSED

    def _on_failure(self) -> None:
        self._failure_count += 1
        self._last_failure_time = time.monotonic()
        _logger.warning(
            "circuit_breaker name=%s failures=%d threshold=%d",
            self.name,
            self._failure_count,
            self.failure_threshold,
        )
        if self._failure_count >= self.failure_threshold:
            if self._state != CircuitState.OPEN:
                _logger.error("circuit_breaker name=%s transition=OPEN", self.name)
            self._state = CircuitState.OPEN
