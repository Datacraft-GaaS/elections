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
