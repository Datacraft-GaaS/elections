from __future__ import annotations

import asyncio
import logging
from typing import Any

_logger = logging.getLogger(__name__)


class BulkheadFull(Exception):
    def __init__(self, name: str) -> None:
        super().__init__(f"Bulkhead '{name}' is full")
        self.name = name


class Bulkhead:
    """Limits concurrent calls via asyncio.Semaphore."""

    def __init__(self, name: str, max_concurrent: int, max_wait_s: float = 5.0) -> None:
        assert max_concurrent > 0, "max_concurrent must be positive"
        self.name = name
        self.max_concurrent = max_concurrent
        self.max_wait_s = max_wait_s
        self._semaphore = asyncio.Semaphore(max_concurrent)
        self._active = 0

    @property
    def available_slots(self) -> int:
        return self._semaphore._value  # type: ignore[attr-defined]

    async def call(self, coro: Any) -> Any:
        try:
            acquired = await asyncio.wait_for(
                self._semaphore.acquire(),
                timeout=self.max_wait_s,
            )
        except asyncio.TimeoutError:
            _logger.warning("bulkhead name=%s full active=%d", self.name, self._active)
            raise BulkheadFull(self.name)

        self._active += 1
        try:
            return await coro
        finally:
            self._active -= 1
            self._semaphore.release()
