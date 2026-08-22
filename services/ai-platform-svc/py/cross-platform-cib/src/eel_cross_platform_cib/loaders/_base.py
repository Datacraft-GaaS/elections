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
