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

"""PII classification wrappers for the eel platform."""

from __future__ import annotations

import asyncio
from enum import Enum
from typing import Any, ClassVar, Generic, Protocol, TypeVar, get_args

from pydantic import GetCoreSchemaHandler
from pydantic_core import core_schema
from sqlalchemy import Text
from sqlalchemy.types import TypeDecorator

__all__ = [
    "Purpose",
    "PIITier",
    "PIIAuditSink",
    "PIIRestricted",
    "PIISensitive",
    "PIIOperational",
    "PIIPublic",
    "PIIEphemeral",
    "pii_column_type",
    "PIIRestrictedType",
    "PIISensitiveType",
    "PIIOperationalType",
    "PIIPublicType",
    "PIIEphemeralType",
]

T = TypeVar("T")


class Purpose(str, Enum):
    OPERATIONS = "operations"
    LEGAL = "legal"
    AUDIT = "audit"
    ANALYTICS = "analytics"
    DEBUG = "debug"
    DISPLAY = "display"


class PIITier(str, Enum):
    RESTRICTED = "A"
    SENSITIVE = "B"
    OPERATIONAL = "C"
    PUBLIC = "D"
    EPHEMERAL = "E"


class PIIAuditSink(Protocol):
    async def record(
        self, tier: PIITier, purpose: Purpose, field: str = ""
    ) -> None: ...


# ---------------------------------------------------------------------------
# Generic PII wrapper base
# ---------------------------------------------------------------------------


class _PIIBase(Generic[T]):
    """Internal base for all PII wrapper tiers."""

    tier: ClassVar[PIITier]
    _label: ClassVar[str]

    def __init__(self, value: T) -> None:
        self._value = value

    def __repr__(self) -> str:
        return f"<{self._label} [REDACTED]>"

    def __str__(self) -> str:
        return f"<{self._label} [REDACTED]>"

    def reveal(self, purpose: Purpose, sink: PIIAuditSink | None = None) -> T:
        if sink is not None:
            try:
                loop = asyncio.get_running_loop()
                loop.create_task(sink.record(self.tier, purpose))
            except RuntimeError:
                pass  # No running loop; skip fire-and-forget
        return self._value

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source_type: Any, handler: GetCoreSchemaHandler
    ) -> core_schema.CoreSchema:
        def validate(v: Any) -> Any:
            if isinstance(v, cls):
                return v
            return cls(v)

        tier_code = cls.tier.value

        return core_schema.no_info_plain_validator_function(
            validate,
            serialization=core_schema.plain_serializer_function_ser_schema(
                lambda obj: f"[REDACTED:{tier_code}]",
                info_arg=False,
            ),
        )


# ---------------------------------------------------------------------------
# Five concrete tiers
# ---------------------------------------------------------------------------


class PIIRestricted(_PIIBase[T]):
    """Tier A: most sensitive — biometrics, national IDs, financial secrets."""

    tier: ClassVar[PIITier] = PIITier.RESTRICTED
    _label: ClassVar[str] = "PIIRestricted"


class PIISensitive(_PIIBase[T]):
    """Tier B: personal identifiers — name, DOB, address."""

    tier: ClassVar[PIITier] = PIITier.SENSITIVE
    _label: ClassVar[str] = "PIISensitive"


class PIIOperational(_PIIBase[T]):
    """Tier C: contact or role info used in service operations."""

    tier: ClassVar[PIITier] = PIITier.OPERATIONAL
    _label: ClassVar[str] = "PIIOperational"


class PIIPublic(_PIIBase[T]):
    """Tier D: non-personal public data."""

    tier: ClassVar[PIITier] = PIITier.PUBLIC
    _label: ClassVar[str] = "PIIPublic"


class PIIEphemeral(_PIIBase[T]):
    """Tier E: session-scoped or transient data."""

    tier: ClassVar[PIITier] = PIITier.EPHEMERAL
    _label: ClassVar[str] = "PIIEphemeral"


# ---------------------------------------------------------------------------
# SQLAlchemy TypeDecorators
# ---------------------------------------------------------------------------


def pii_column_type(tier: PIITier, inner_type: type = str) -> type[TypeDecorator[Any]]:
    """Return a SQLAlchemy TypeDecorator that stores PII with a tier prefix."""
    prefix = f"pii:{tier.value}:"

    class _PIIColumnType(TypeDecorator[Any]):
        impl = Text
        cache_ok = True

        def process_bind_param(self, value: Any, dialect: Any) -> str | None:
            if value is None:
                return None
            raw = value._value if isinstance(value, _PIIBase) else value
            return f"{prefix}{raw}"

        def process_result_value(self, value: str | None, dialect: Any) -> Any:
            if value is None:
                return None
            if value.startswith(prefix):
                return inner_type(value[len(prefix) :])
            return value

    _PIIColumnType.__name__ = f"PII{tier.name.title()}ColumnType"
    _PIIColumnType.__qualname__ = _PIIColumnType.__name__
    return _PIIColumnType


PIIRestrictedType = pii_column_type(PIITier.RESTRICTED)
PIISensitiveType = pii_column_type(PIITier.SENSITIVE)
PIIOperationalType = pii_column_type(PIITier.OPERATIONAL)
PIIPublicType = pii_column_type(PIITier.PUBLIC)
PIIEphemeralType = pii_column_type(PIITier.EPHEMERAL)
