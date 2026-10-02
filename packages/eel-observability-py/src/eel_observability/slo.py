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

import time
from typing import Any

from opentelemetry import metrics


def record_latency(
    meter: metrics.Meter,
    name: str,
    value_ms: float,
    labels: dict[str, str] | None = None,
) -> None:
    """Record a latency observation to a histogram."""
    histogram = meter.create_histogram(
        name=f"{name}.latency_ms",
        description="Latency in milliseconds",
        unit="ms",
    )
    histogram.record(value_ms, attributes=labels or {})


def record_error(
    meter: metrics.Meter,
    name: str,
    labels: dict[str, str] | None = None,
) -> None:
    """Increment an error counter."""
    counter = meter.create_counter(
        name=f"{name}.errors",
        description="Error count",
    )
    counter.add(1, attributes=labels or {})


def record_success(
    meter: metrics.Meter,
    name: str,
    labels: dict[str, str] | None = None,
) -> None:
    """Increment a success counter."""
    counter = meter.create_counter(
        name=f"{name}.successes",
        description="Success count",
    )
    counter.add(1, attributes=labels or {})


class SloTimer:
    """Context manager that records latency on exit."""

    def __init__(
        self,
        meter: metrics.Meter,
        name: str,
        labels: dict[str, str] | None = None,
    ) -> None:
        self._meter = meter
        self._name = name
        self._labels = labels
        self._start: float = 0.0

    async def __aenter__(self) -> SloTimer:
        self._start = time.monotonic()
        return self

    async def __aexit__(self, *_: Any) -> None:
        elapsed_ms = (time.monotonic() - self._start) * 1000.0
        record_latency(self._meter, self._name, elapsed_ms, self._labels)
