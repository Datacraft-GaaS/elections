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

"""Protocols for the eel event store."""

from __future__ import annotations

from typing import Protocol

from eel_eventstore.models import AggregateSnapshot, StoredEvent


class Aggregate(Protocol):
    stream_id: str
    seq: int

    def apply(self, event: StoredEvent) -> None:
        """Apply a stored event to advance aggregate state."""
        ...

    def snapshot(self) -> AggregateSnapshot:
        """Return a point-in-time snapshot of the aggregate state."""
        ...
