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

"""Hybrid Logical Clocks (HLC) for the eel platform."""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Protocol, runtime_checkable

__all__ = ["Hlc", "HlcClock", "HlcClockProtocol"]

_COUNTER_MAX = 65535


@dataclass(eq=False)
class Hlc:
    """HLC timestamp with wire form ``physical_ms.counter.node``."""

    physical_ms: int
    counter: int
    node: int

    @classmethod
    def from_str(cls, s: str) -> "Hlc":
        parts = s.split(".")
        if len(parts) != 3:
            raise ValueError(
                f"Invalid HLC string: {s!r} — expected 'physical_ms.counter.node'"
            )
        return cls(
            physical_ms=int(parts[0]),
            counter=int(parts[1]),
            node=int(parts[2]),
        )

    def __str__(self) -> str:
        return f"{self.physical_ms}.{self.counter}.{self.node}"

    def _tuple(self) -> tuple[int, int, int]:
        return (self.physical_ms, self.counter, self.node)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Hlc):
            return NotImplemented
        return self._tuple() == other._tuple()

    def __lt__(self, other: "Hlc") -> bool:
        return self._tuple() < other._tuple()

    def __le__(self, other: "Hlc") -> bool:
        return self._tuple() <= other._tuple()

    def __gt__(self, other: "Hlc") -> bool:
        return self._tuple() > other._tuple()

    def __ge__(self, other: "Hlc") -> bool:
        return self._tuple() >= other._tuple()

    def __hash__(self) -> int:
        return hash(self._tuple())

    def merge(self, other: "Hlc") -> "Hlc":
        """Return the greater of self and other (for receive-side merging)."""
        return self if self >= other else other


class HlcClock:
    """Wall-clock-aware, monotonic HLC clock for a given node ID."""

    def __init__(self, node: int) -> None:
        self._node = node
        self._last_phys: int = 0
        self._last_counter: int = 0

    def _wall_ms(self) -> int:
        return int(time.time() * 1000)

    def now(self) -> Hlc:
        """Advance the clock and return the next timestamp."""
        wall = self._wall_ms()
        phys = max(wall, self._last_phys)
        if phys == self._last_phys:
            counter = self._last_counter + 1
            if counter > _COUNTER_MAX:
                raise ValueError(f"HLC counter overflow at physical_ms={phys}")
        else:
            counter = 0
        self._last_phys = phys
        self._last_counter = counter
        return Hlc(physical_ms=phys, counter=counter, node=self._node)

    def receive(self, remote: Hlc) -> Hlc:
        """Merge remote timestamp into local state and return the new timestamp."""
        wall = self._wall_ms()
        phys = max(wall, self._last_phys, remote.physical_ms)
        if phys == self._last_phys == remote.physical_ms:
            counter = max(self._last_counter, remote.counter) + 1
        elif phys == self._last_phys:
            counter = self._last_counter + 1
        elif phys == remote.physical_ms:
            counter = remote.counter + 1
        else:
            counter = 0
        if counter > _COUNTER_MAX:
            raise ValueError(f"HLC counter overflow at physical_ms={phys}")
        self._last_phys = phys
        self._last_counter = counter
        return Hlc(physical_ms=phys, counter=counter, node=self._node)


@runtime_checkable
class HlcClockProtocol(Protocol):
    """Structural protocol for HLC clock implementations."""

    def now(self) -> Hlc: ...
    def receive(self, remote: Hlc) -> Hlc: ...
