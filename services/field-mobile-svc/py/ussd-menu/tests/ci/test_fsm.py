"""USSD FSM unit tests (WP-4.7, M13).

Tests: FR-M13-024, FR-M13-027, FR-M13-029, AC-M13-03.
"""

import pytest

from ussd_menu.fsm import (
    _GRIEVANCE_CONFIRM,
    _GRIEVANCE_DONE,
    _GRIEVANCE_TYPE,
    _MAIN_MENU,
    _RESULTS_PROMPT,
    _TOKEN_PART1,
    _TOKEN_PART2,
    parse_steps,
    route_menu,
)
from ussd_menu.views import USSD_MAX_CHARS


@pytest.mark.trace("FR-M13-029", "AC-M13-03")
def test_all_static_pages_within_182_char_budget():
    """All static USSD pages must be within the 182-char budget."""
    pages = [
        _MAIN_MENU,
        _TOKEN_PART1,
        _TOKEN_PART2,
        _RESULTS_PROMPT,
        _GRIEVANCE_TYPE,
        _GRIEVANCE_CONFIRM,
        _GRIEVANCE_DONE,
    ]
    for page in pages:
        assert len(page) <= USSD_MAX_CHARS, (
            f"Page exceeds {USSD_MAX_CHARS} chars ({len(page)}): {page!r}"
        )


@pytest.mark.trace("FR-M13-024", "FR-M13-027")
def test_main_menu_returns_con():
    text, is_final = route_menu([])
    assert not is_final
    assert "1" in text
    assert "2" in text
    assert "3" in text


@pytest.mark.trace("FR-M13-024", "AC-M13-03")
def test_verify_token_flow_within_5_steps():
    """Token verify completes in exactly 3 steps (<=5 required)."""
    # Step 1: choose verify
    text1, final1 = route_menu(["1"])
    assert not final1

    # Step 2: enter first 8 digits
    text2, final2 = route_menu(["1", "12345678"])
    assert not final2

    # Step 3: enter last 8 digits -> terminal
    text3, final3 = route_menu(["1", "12345678", "ABCDEFGH"])
    assert final3
    assert len(text3) <= USSD_MAX_CHARS


@pytest.mark.trace("FR-M13-024")
def test_grievance_flow():
    """Grievance path completes within 5 steps."""
    _, f = route_menu(["3"])
    assert not f

    _, f = route_menu(["3", "1"])
    assert not f

    text, f = route_menu(["3", "1", "1"])
    assert f
    assert len(text) <= USSD_MAX_CHARS


def test_results_flow():
    _, f = route_menu(["2"])
    assert not f

    text, f = route_menu(["2", "047"])
    assert f
    assert len(text) <= USSD_MAX_CHARS


def test_exit_is_final():
    text, f = route_menu(["4"])
    assert f


def test_parse_steps_empty():
    assert parse_steps("") == []


def test_parse_steps_multi():
    assert parse_steps("1*12345678") == ["1", "12345678"]
