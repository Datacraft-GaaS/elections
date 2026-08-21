from __future__ import annotations

import asyncio

import pytest

from eel_resilience.bulkhead import Bulkhead, BulkheadFull
from eel_resilience.circuit_breaker import (
    CircuitBreaker,
    CircuitBreakerOpen,
    CircuitState,
)
from eel_resilience.degradation import DegradationFlag, DegradationLevel, DegradedError
from eel_resilience.idempotency import InMemoryIdempotencyStore
from eel_resilience.retry import RetryBudgetExhausted, RetryConfig, retry_with_jitter
from eel_resilience.timeout import with_timeout


# --- CircuitBreaker ---


async def test_circuit_breaker_opens_after_threshold():
    cb = CircuitBreaker("test", failure_threshold=5, recovery_timeout_s=60.0)
    assert cb.state == CircuitState.CLOSED

    async def fail():
        raise ValueError("boom")

    for _ in range(5):
        with pytest.raises(ValueError):
            await cb.call(fail())

    assert cb.state == CircuitState.OPEN

    with pytest.raises(CircuitBreakerOpen):
        await cb.call(fail())


async def test_circuit_breaker_half_open_after_timeout():
    cb = CircuitBreaker("test2", failure_threshold=2, recovery_timeout_s=0.01)

    async def fail():
        raise ValueError("boom")

    for _ in range(2):
        with pytest.raises(ValueError):
            await cb.call(fail())

    assert cb._state == CircuitState.OPEN

    await asyncio.sleep(0.02)
    # accessing .state should transition to HALF_OPEN
    assert cb.state == CircuitState.HALF_OPEN


async def test_circuit_breaker_recovers():
    cb = CircuitBreaker("test3", failure_threshold=1, recovery_timeout_s=0.01)

    async def fail():
        raise ValueError("boom")

    async def succeed():
        return "ok"

    with pytest.raises(ValueError):
        await cb.call(fail())

    await asyncio.sleep(0.02)
    result = await cb.call(succeed())
    assert result == "ok"
    assert cb.state == CircuitState.CLOSED


# --- Bulkhead ---


async def test_bulkhead_blocks_when_full():
    bh = Bulkhead("test", max_concurrent=1, max_wait_s=0.05)

    event = asyncio.Event()

    async def hold():
        await event.wait()

    # Start a coroutine that holds the slot
    task = asyncio.create_task(bh.call(hold()))
    await asyncio.sleep(0.01)  # let it acquire

    # Second call should time out
    with pytest.raises(BulkheadFull):
        await bh.call(hold())

    event.set()
    await task


async def test_bulkhead_available_slots():
    bh = Bulkhead("test2", max_concurrent=3, max_wait_s=0.1)
    assert bh.available_slots == 3


# --- retry_with_jitter ---


async def test_retry_succeeds_on_third_attempt():
    attempts = 0

    async def sometimes_fail():
        nonlocal attempts
        attempts += 1
        if attempts < 3:
            raise ValueError(f"attempt {attempts}")
        return "done"

    config = RetryConfig(max_attempts=3, base_delay_s=0.0, jitter=False)
    result = await retry_with_jitter(sometimes_fail, config)
    assert result == "done"
    assert attempts == 3


async def test_retry_exhausted():
    async def always_fail():
        raise RuntimeError("always")

    config = RetryConfig(max_attempts=2, base_delay_s=0.0, jitter=False)
    with pytest.raises(RetryBudgetExhausted) as exc_info:
        await retry_with_jitter(always_fail, config)
    assert exc_info.value.attempts == 2


async def test_retry_non_retryable_exception_not_retried():
    calls = 0

    async def fail():
        nonlocal calls
        calls += 1
        raise KeyError("nope")

    config = RetryConfig(
        max_attempts=3,
        base_delay_s=0.0,
        jitter=False,
        retryable_exceptions=(ValueError,),
    )
    with pytest.raises(KeyError):
        await retry_with_jitter(fail, config)
    assert calls == 1


# --- with_timeout ---


async def test_timeout_raises():
    async def slow():
        await asyncio.sleep(10)

    with pytest.raises(asyncio.TimeoutError):
        await with_timeout(slow(), timeout_s=0.01, name="slow_op")


async def test_timeout_passes_through():
    async def fast():
        return 42

    result = await with_timeout(fast(), timeout_s=1.0)
    assert result == 42


# --- InMemoryIdempotencyStore ---


async def test_idempotency_store_get_returns_none_initially():
    store = InMemoryIdempotencyStore()
    result = await store.get("missing-key")
    assert result is None


async def test_idempotency_store_set_and_get():
    store = InMemoryIdempotencyStore()
    await store.set("key1", 200, b'{"ok":true}', b"hash123")
    result = await store.get("key1")
    assert result is not None
    status, body, body_hash = result
    assert status == 200
    assert body == b'{"ok":true}'
    assert body_hash == b"hash123"


# --- DegradationFlag ---


def test_degradation_flag_starts_normal():
    flag = DegradationFlag()
    assert flag.level == DegradationLevel.NORMAL
    assert not flag.is_degraded()


def test_degradation_flag_require_normal_raises():
    flag = DegradationFlag(DegradationLevel.DEGRADED)
    with pytest.raises(DegradedError) as exc_info:
        flag.require_normal()
    assert exc_info.value.level == DegradationLevel.DEGRADED


def test_degradation_flag_require_normal_passes():
    flag = DegradationFlag(DegradationLevel.NORMAL)
    flag.require_normal()  # should not raise


def test_degradation_flag_set():
    flag = DegradationFlag()
    flag.set(DegradationLevel.EMERGENCY)
    assert flag.level == DegradationLevel.EMERGENCY
    assert flag.is_degraded()
