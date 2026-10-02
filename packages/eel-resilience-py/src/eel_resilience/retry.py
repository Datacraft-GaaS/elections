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
import random
from typing import Any, Callable, Coroutine

_logger = logging.getLogger(__name__)


class RetryBudgetExhausted(Exception):
    def __init__(self, attempts: int, last_exc: Exception) -> None:
        super().__init__(
            f"Retry budget exhausted after {attempts} attempts: {last_exc}"
        )
        self.attempts = attempts
        self.last_exc = last_exc


class RetryConfig:
    def __init__(
        self,
        max_attempts: int = 3,
        base_delay_s: float = 0.1,
        max_delay_s: float = 30.0,
        jitter: bool = True,
        retryable_exceptions: tuple[type[Exception], ...] = (Exception,),
    ) -> None:
        assert max_attempts >= 1, "max_attempts must be >= 1"
        assert base_delay_s >= 0, "base_delay_s must be non-negative"
        assert max_delay_s >= base_delay_s, "max_delay_s must be >= base_delay_s"
        self.max_attempts = max_attempts
        self.base_delay_s = base_delay_s
        self.max_delay_s = max_delay_s
        self.jitter = jitter
        self.retryable_exceptions = retryable_exceptions


async def retry_with_jitter(
    coro_factory: Callable[[], Coroutine[Any, Any, Any]],
    config: RetryConfig,
) -> Any:
    """Retry with exponential backoff and full jitter."""
    last_exc: Exception | None = None
    for attempt in range(1, config.max_attempts + 1):
        try:
            return await coro_factory()
        except config.retryable_exceptions as exc:
            last_exc = exc
            if attempt == config.max_attempts:
                break
            cap = min(config.max_delay_s, config.base_delay_s * (2 ** (attempt - 1)))
            delay = random.uniform(0, cap) if config.jitter else cap
            _logger.warning(
                "retry attempt=%d/%d delay_s=%.3f exc=%s",
                attempt,
                config.max_attempts,
                delay,
                exc,
            )
            await asyncio.sleep(delay)

    assert last_exc is not None
    raise RetryBudgetExhausted(config.max_attempts, last_exc)
