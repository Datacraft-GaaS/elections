from __future__ import annotations

import threading
from enum import Enum


class DegradationLevel(Enum):
    NORMAL = "normal"
    DEGRADED = "degraded"
    EMERGENCY = "emergency"
    READONLY = "readonly"


class DegradedError(Exception):
    def __init__(self, level: DegradationLevel) -> None:
        super().__init__(f"Service is in degraded state: {level.value}")
        self.level = level


class DegradationFlag:
    """Thread-safe flag for service degradation mode."""

    def __init__(self, initial: DegradationLevel = DegradationLevel.NORMAL) -> None:
        self._level = initial
        self._lock = threading.Lock()

    @property
    def level(self) -> DegradationLevel:
        with self._lock:
            return self._level

    def set(self, level: DegradationLevel) -> None:
        with self._lock:
            self._level = level

    def is_degraded(self) -> bool:
        return self.level != DegradationLevel.NORMAL

    def require_normal(self) -> None:
        """Raise DegradedError if not NORMAL."""
        current = self.level
        if current != DegradationLevel.NORMAL:
            raise DegradedError(current)
