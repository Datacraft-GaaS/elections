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

"""Unified Post schema for cross-platform CIB analysis."""

from __future__ import annotations

import hashlib
from datetime import datetime
from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator


class EngagementStats(BaseModel):
    model_config = ConfigDict(extra="forbid")

    likes: int = 0
    shares: int = 0
    comments: int = 0
    views: int = 0
    reactions: dict[str, int] = Field(default_factory=dict)


class Post(BaseModel):
    """Unified post representation across all supported platforms.

    Fields are normalised so a downstream graph builder can treat posts from
    any platform identically.
    """

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    platform: str = Field(
        description="Source platform identifier (facebook|twitter|tiktok|whatsapp_broadcast|telegram)"
    )
    author_id: str = Field(
        description="Platform-scoped author identifier (hashed if PII)"
    )
    content_hash: str = Field(description="SHA-256 of normalised post text/media bytes")
    ts: datetime = Field(description="Publication timestamp (UTC)")
    engagement_stats: EngagementStats = Field(default_factory=EngagementStats)
    shared_from: str | None = Field(
        default=None,
        description="content_hash of the post this was shared/forwarded from",
    )
    post_id: str | None = Field(
        default=None, description="Platform-native post ID (optional)"
    )
    language: str | None = Field(
        default=None, description="BCP-47 language tag if known"
    )

    @field_validator("platform")
    @classmethod
    def validate_platform(cls, v: str) -> str:
        allowed = {"facebook", "twitter", "tiktok", "whatsapp_broadcast", "telegram"}
        if v not in allowed:
            raise ValueError(f"platform must be one of {allowed}, got {v!r}")
        return v

    @field_validator("content_hash")
    @classmethod
    def validate_content_hash(cls, v: str) -> str:
        if len(v) != 64:
            raise ValueError("content_hash must be a 64-character hex SHA-256 digest")
        return v.lower()


def make_content_hash(text: str) -> str:
    """Compute the canonical content hash for a normalised text string."""
    normalised = text.strip().lower()
    return hashlib.sha256(normalised.encode()).hexdigest()
