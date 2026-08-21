# eel-resilience-py

Resilience primitives for the eel platform: circuit breaker, bulkhead, retry with jitter, timeout, idempotency middleware, and degradation flags.

## Modules

- `circuit_breaker` - CircuitBreaker with CLOSED/OPEN/HALF_OPEN states
- `bulkhead` - Semaphore-based concurrency limiter
- `retry` - Exponential backoff with full jitter
- `timeout` - asyncio.wait_for wrapper
- `idempotency` - Starlette middleware + pluggable store
- `degradation` - Thread-safe degradation level flag

## Install

```
uv add eel-resilience-py
```

## Test

```
uv run pytest tests/ci -x -q
```
