from __future__ import annotations

import asyncio
import logging
from typing import Any

_logger = logging.getLogger(__name__)


async def with_timeout(coro: Any, timeout_s: float, name: str = "") -> Any:
    """Wrap coroutine with asyncio.wait_for. Raises asyncio.TimeoutError."""
    try:
        return await asyncio.wait_for(coro, timeout=timeout_s)
    except asyncio.TimeoutError:
        _logger.warning("timeout name=%s timeout_s=%.3f", name, timeout_s)
        raise
