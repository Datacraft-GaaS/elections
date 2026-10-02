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

"""eel-resilience: circuit breaker, bulkhead, retry, timeout, idempotency, and degradation."""

from eel_resilience.bulkhead import Bulkhead, BulkheadFull
from eel_resilience.circuit_breaker import (
    CircuitBreaker,
    CircuitBreakerOpen,
    CircuitState,
)
from eel_resilience.degradation import DegradationFlag, DegradationLevel, DegradedError
from eel_resilience.idempotency import (
    IdempotencyMiddleware,
    IdempotencyStore,
    InMemoryIdempotencyStore,
)
from eel_resilience.retry import RetryBudgetExhausted, RetryConfig, retry_with_jitter
from eel_resilience.timeout import with_timeout

__all__ = [
    "CircuitBreaker",
    "CircuitState",
    "CircuitBreakerOpen",
    "Bulkhead",
    "BulkheadFull",
    "RetryConfig",
    "retry_with_jitter",
    "RetryBudgetExhausted",
    "with_timeout",
    "IdempotencyMiddleware",
    "IdempotencyStore",
    "InMemoryIdempotencyStore",
    "DegradationLevel",
    "DegradationFlag",
    "DegradedError",
]
