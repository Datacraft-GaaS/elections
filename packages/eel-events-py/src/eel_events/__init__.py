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

"""eel-events-py: CloudEvents envelope, signing, and consumer middleware."""

from eel_events.builder import EventBuilder
from eel_events.consumer import ConsumerMiddleware, DedupStore, EventConsumer
from eel_events.envelope import DataClass, Envelope
from eel_events.karapace import KarapaceClient
from eel_events.rfc8785 import canonicalize
from eel_events.signing import SpiffeKeyProvider

__all__ = [
    "Envelope",
    "DataClass",
    "EventBuilder",
    "SpiffeKeyProvider",
    "KarapaceClient",
    "EventConsumer",
    "DedupStore",
    "ConsumerMiddleware",
    "canonicalize",
]
