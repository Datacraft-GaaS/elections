"""Tests for eel-pii-py."""

import asyncio
from typing import Any

import pytest
from pydantic import BaseModel

from eel_pii import (
    PIIAuditSink,
    PIIEphemeral,
    PIIOperational,
    PIIPublic,
    PIIRestricted,
    PIISensitive,
    PIITier,
    Purpose,
)


# --- repr / str redaction ---


def test_repr_redacts_restricted() -> None:
    w = PIIRestricted("secret")
    assert "secret" not in repr(w)
    assert "REDACTED" in repr(w)
    assert "PIIRestricted" in repr(w)


def test_str_redacts_all_tiers() -> None:
    for cls in (PIIRestricted, PIISensitive, PIIOperational, PIIPublic, PIIEphemeral):
        w = cls("value")
        assert "value" not in str(w)
        assert "REDACTED" in str(w)


# --- reveal ---


def test_reveal_returns_value() -> None:
    w = PIIRestricted(42)
    assert w.reveal(Purpose.OPERATIONS) == 42


def test_reveal_works_for_all_tiers() -> None:
    cases = [
        (PIIRestricted, PIITier.RESTRICTED),
        (PIISensitive, PIITier.SENSITIVE),
        (PIIOperational, PIITier.OPERATIONAL),
        (PIIPublic, PIITier.PUBLIC),
        (PIIEphemeral, PIITier.EPHEMERAL),
    ]
    for cls, expected_tier in cases:
        w = cls("x")
        assert w.reveal(Purpose.DISPLAY) == "x"
        assert w.tier == expected_tier


# --- Pydantic model serialization redacts ---


class _Model(BaseModel):
    national_id: PIIRestricted[str]
    name: PIISensitive[str]


def test_pydantic_model_dump_redacts() -> None:
    m = _Model(national_id=PIIRestricted("123456"), name=PIISensitive("Alice"))
    d = m.model_dump()
    assert d["national_id"] == "[REDACTED:A]"
    assert d["name"] == "[REDACTED:B]"


def test_pydantic_model_json_redacts() -> None:
    m = _Model(national_id=PIIRestricted("123456"), name=PIISensitive("Alice"))
    j = m.model_dump_json()
    assert "123456" not in j
    assert "Alice" not in j
    assert "[REDACTED:A]" in j
    assert "[REDACTED:B]" in j


# --- Pydantic validation wraps raw values ---


def test_pydantic_validation_wraps_raw_value() -> None:
    m = _Model(national_id="raw-id", name="raw-name")  # type: ignore[arg-type]
    assert isinstance(m.national_id, PIIRestricted)
    assert isinstance(m.name, PIISensitive)
    # reveal still works after wrapping
    assert m.national_id.reveal(Purpose.LEGAL) == "raw-id"
    assert m.name.reveal(Purpose.LEGAL) == "raw-name"


def test_pydantic_validation_accepts_wrapper_directly() -> None:
    m = _Model(national_id=PIIRestricted("abc"), name=PIISensitive("Bob"))
    assert m.national_id.reveal(Purpose.AUDIT) == "abc"


# --- audit sink is called on reveal ---


async def test_audit_sink_called_on_reveal() -> None:
    records: list[tuple[PIITier, Purpose, str]] = []

    class _Sink:
        async def record(
            self, tier: PIITier, purpose: Purpose, field: str = ""
        ) -> None:
            records.append((tier, purpose, field))

    sink = _Sink()
    w = PIIRestricted("biometric")
    result = w.reveal(Purpose.AUDIT, sink=sink)
    assert result == "biometric"
    # Let the fire-and-forget task run
    await asyncio.sleep(0)
    assert len(records) == 1
    assert records[0][0] == PIITier.RESTRICTED
    assert records[0][1] == Purpose.AUDIT


async def test_audit_sink_called_for_each_tier() -> None:
    records: list[PIITier] = []

    class _Sink:
        async def record(
            self, tier: PIITier, purpose: Purpose, field: str = ""
        ) -> None:
            records.append(tier)

    sink = _Sink()
    wrappers = [
        PIIRestricted("a"),
        PIISensitive("b"),
        PIIOperational("c"),
        PIIPublic("d"),
        PIIEphemeral("e"),
    ]
    for w in wrappers:
        w.reveal(Purpose.OPERATIONS, sink=sink)

    await asyncio.sleep(0)
    assert len(records) == 5
    assert set(records) == {
        PIITier.RESTRICTED,
        PIITier.SENSITIVE,
        PIITier.OPERATIONAL,
        PIITier.PUBLIC,
        PIITier.EPHEMERAL,
    }


def test_reveal_without_sink_no_error() -> None:
    w = PIISensitive("test")
    assert w.reveal(Purpose.DEBUG) == "test"
