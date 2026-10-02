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

"""Base loader protocol and helpers."""

from __future__ import annotations

from typing import Any, Protocol

from ..models import Post


class PlatformLoader(Protocol):
    """Protocol that all platform loaders must satisfy."""

    platform: str

    def load(self, raw: dict[str, Any]) -> Post:
        """Transform a raw platform-native record into a unified Post."""
        ...

    def load_batch(self, records: list[dict[str, Any]]) -> list[Post]:
        """Transform a list of raw records."""
        ...
