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

"""Tests for eel-hlc-py."""

import asyncio

import pytest

from eel_hlc import Hlc, HlcClock, HlcClockProtocol


# --- parse / round-trip ---


def test_parse_and_roundtrip() -> None:
    s = "1723731000123.5.42"
    h = Hlc.from_str(s)
    assert h.physical_ms == 1723731000123
    assert h.counter == 5
    assert h.node == 42
    assert str(h) == s


def test_parse_zero_counter() -> None:
    h = Hlc.from_str("1000.0.1")
    assert h.counter == 0
    assert str(h) == "1000.0.1"


def test_parse_invalid_raises() -> None:
    with pytest.raises(ValueError):
        Hlc.from_str("bad.input")


# --- ordering ---


def test_ordering_by_physical_ms() -> None:
    a = Hlc(1000, 0, 1)
    b = Hlc(2000, 0, 1)
    assert a < b
    assert b > a
    assert a <= b
    assert b >= a
    assert a != b


def test_ordering_by_counter_when_same_physical() -> None:
    a = Hlc(1000, 0, 1)
    b = Hlc(1000, 1, 1)
    assert a < b


def test_ordering_by_node_when_same_physical_and_counter() -> None:
    a = Hlc(1000, 0, 1)
    b = Hlc(1000, 0, 2)
    assert a < b


def test_equality() -> None:
    a = Hlc(1000, 5, 3)
    b = Hlc(1000, 5, 3)
    assert a == b
    assert not (a != b)


# --- merge ---


def test_merge_returns_greater() -> None:
    a = Hlc(1000, 0, 1)
    b = Hlc(2000, 0, 1)
    assert a.merge(b) == b
    assert b.merge(a) == b


def test_merge_equal_returns_self() -> None:
    a = Hlc(1000, 5, 1)
    b = Hlc(1000, 5, 1)
    result = a.merge(b)
    assert result == a


# --- HlcClock.now() monotonicity ---


def test_clock_now_is_monotonic() -> None:
    clock = HlcClock(node=1)
    prev = clock.now()
    for _ in range(99):
        curr = clock.now()
        assert curr > prev, f"{curr} not > {prev}"
        prev = curr


def test_clock_now_increments_counter_on_same_ms() -> None:
    clock = HlcClock(node=1)
    # Force the internal physical to a fixed value so counter increments
    clock._last_phys = 10**15  # far future, wall will always be less
    clock._last_counter = 0
    a = clock.now()
    b = clock.now()
    assert b.counter == a.counter + 1


def test_clock_counter_overflow_raises() -> None:
    clock = HlcClock(node=1)
    clock._last_phys = 10**15
    clock._last_counter = 65535
    with pytest.raises(ValueError, match="overflow"):
        clock.now()


# --- HlcClock.receive() ---


def test_receive_advances_clock_past_remote() -> None:
    clock = HlcClock(node=1)
    remote = Hlc(physical_ms=10**15, counter=10, node=2)
    result = clock.receive(remote)
    # Physical must be at least remote's
    assert result.physical_ms >= remote.physical_ms
    # Counter must be strictly greater than remote's when physical matches
    if result.physical_ms == remote.physical_ms:
        assert result.counter > remote.counter


def test_receive_monotonic_after_receive() -> None:
    clock = HlcClock(node=1)
    remote = Hlc(physical_ms=10**15, counter=0, node=2)
    r1 = clock.receive(remote)
    r2 = clock.now()
    assert r2 > r1


# --- protocol conformance ---


def test_hlc_clock_satisfies_protocol() -> None:
    clock = HlcClock(node=99)
    assert isinstance(clock, HlcClockProtocol)
