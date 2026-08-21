from __future__ import annotations

import logging
from unittest.mock import MagicMock

import pytest

from eel_observability.audit_shadow import generate_audit_shadow_trigger
from eel_observability.log_scrubber import PiiScrubberFilter, _scrub
from eel_observability.slo import SloTimer


# --- _scrub ---


def test_scrub_email():
    result = _scrub("Contact user@example.com for details")
    assert "[EMAIL_REDACTED]" in result
    assert "user@example.com" not in result


def test_scrub_national_id():
    result = _scrub("ID number 12345678 is registered")
    assert "[ID_REDACTED]" in result
    assert "12345678" not in result


def test_scrub_card_number():
    result = _scrub("Card 4111 1111 1111 1111 expired")
    assert "[CARD_REDACTED]" in result
    assert "4111 1111 1111 1111" not in result


def test_scrub_card_no_spaces():
    result = _scrub("Card 4111111111111111 expired")
    assert "[CARD_REDACTED]" in result


def test_scrub_leaves_short_numbers():
    result = _scrub("Version 3.14 and code 42")
    assert result == "Version 3.14 and code 42"


def test_scrub_multiple_patterns():
    result = _scrub("Email user@test.org and ID 987654321")
    assert "[EMAIL_REDACTED]" in result
    assert "[ID_REDACTED]" in result


# --- PiiScrubberFilter ---


def test_pii_filter_modifies_msg():
    f = PiiScrubberFilter()
    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="user email is admin@company.com",
        args=(),
        exc_info=None,
    )
    f.filter(record)
    assert "[EMAIL_REDACTED]" in record.msg
    assert "admin@company.com" not in record.msg


def test_pii_filter_returns_true():
    f = PiiScrubberFilter()
    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="safe message",
        args=(),
        exc_info=None,
    )
    assert f.filter(record) is True


# --- generate_audit_shadow_trigger ---


def test_audit_shadow_contains_table_name():
    sql = generate_audit_shadow_trigger("voters")
    assert "voters" in sql


def test_audit_shadow_contains_history_table():
    sql = generate_audit_shadow_trigger("ballots")
    assert "audit_ballots_history" in sql


def test_audit_shadow_contains_operations():
    sql = generate_audit_shadow_trigger("results")
    assert "INSERT" in sql
    assert "UPDATE" in sql
    assert "DELETE" in sql


def test_audit_shadow_custom_schema():
    sql = generate_audit_shadow_trigger("votes", schema="election")
    assert "election" in sql
    assert "election.audit_votes_history" in sql


# --- SloTimer ---


async def test_slo_timer_no_exception():
    mock_meter = MagicMock()
    mock_histogram = MagicMock()
    mock_meter.create_histogram.return_value = mock_histogram

    async with SloTimer(mock_meter, "test_op"):
        pass  # just verify it completes

    mock_meter.create_histogram.assert_called_once()
    mock_histogram.record.assert_called_once()


async def test_slo_timer_records_positive_latency():
    import asyncio

    mock_meter = MagicMock()
    mock_histogram = MagicMock()
    mock_meter.create_histogram.return_value = mock_histogram

    async with SloTimer(mock_meter, "latency_test"):
        await asyncio.sleep(0.01)

    args, kwargs = mock_histogram.record.call_args
    latency_ms = args[0]
    assert latency_ms >= 0
