# eel-resilience-py

Circuit breaker, bulkhead, retry, timeout, idempotency, and degradation primitives for eel Python services. Mirror of `eel-resilience-rs`.

## Purpose in context

Python FastAPI services must expose the same degradation modes as their Rust counterparts. This package provides async-native Python implementations of all resilience primitives, plus `DegradationFlag` for coordinating Normal / Degraded / ReadOnlyLockdown state across handlers.

## Public API

```python
from eel_resilience import (
    CircuitBreaker, CircuitState, CircuitBreakerOpen,
    Bulkhead, BulkheadFull,
    RetryConfig, RetryBudgetExhausted, retry_with_jitter,
    with_timeout,
    IdempotencyMiddleware, IdempotencyStore, InMemoryIdempotencyStore,
    DegradationFlag, DegradationLevel, DegradedError,
)
```

| Symbol | Description |
|---|---|
| `CircuitBreaker` | Async circuit breaker; `call(coro)` or use as async context manager |
| `Bulkhead` | Async semaphore-based concurrency limiter |
| `retry_with_jitter(config, coro_fn)` | Jittered exponential retry |
| `with_timeout(seconds, coro)` | Raises `asyncio.TimeoutError` on breach |
| `IdempotencyMiddleware` | Starlette middleware; deduplicates by `Idempotency-Key` header |
| `DegradationFlag` | Shared async-safe mode flag |
| `DegradationLevel` | `Normal | Degraded | ReadOnlyLockdown` |

## Layout

```
eel-resilience-py/
  src/eel_resilience/
    __init__.py
    circuit_breaker.py
    bulkhead.py
    retry.py
    timeout.py
    idempotency.py
    degradation.py
  tests/
  pyproject.toml
```

## Usage

```python
import asyncio
from eel_resilience import CircuitBreaker, RetryConfig, retry_with_jitter, DegradationFlag, DegradationLevel

cb = CircuitBreaker(failure_threshold=5, recovery_timeout=30)

async def call_external():
    async with cb:
        return await some_external_api()

flag = DegradationFlag()
if await flag.level() == DegradationLevel.ReadOnlyLockdown:
    raise DegradedError("service is in read-only mode")

cfg = RetryConfig(max_attempts=3, base_ms=100)
result = await retry_with_jitter(cfg, lambda: fetch_data())
```

## Dependencies

`starlette>=0.40`, `pydantic>=2.0`.

## Install

```bash
uv add eel-resilience-py
```

## Test

```bash
uv run pytest packages/eel-resilience-py/tests -v
uv run pyright packages/eel-resilience-py/src
```

## Related docs

- `packages/eel-resilience-rs/README.md` - Rust canonical implementation
- `docs/adr/0009-resilience.md` - resilience patterns and SLO targets
- `docs/runbooks/<svc>-degradation.md` - per-service degradation runbooks

## Limitations

`DegradationFlag` is process-local. Cluster-wide coordination requires a Postgres advisory lock; the runbook for each service describes the operator procedure.
