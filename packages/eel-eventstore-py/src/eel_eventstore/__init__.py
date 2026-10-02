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

"""eel-eventstore-py: Append-only event store with optimistic concurrency."""

from eel_eventstore.models import AggregateSnapshot, StoredEvent
from eel_eventstore.protocols import Aggregate
from eel_eventstore.store import (
    EventStore,
    EventStoreSchema,
    OptimisticConcurrencyError,
)

__all__ = [
    "StoredEvent",
    "AggregateSnapshot",
    "Aggregate",
    "EventStore",
    "OptimisticConcurrencyError",
    "EventStoreSchema",
]
