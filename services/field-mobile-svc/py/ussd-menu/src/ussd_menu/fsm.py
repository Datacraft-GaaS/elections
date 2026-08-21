"""USSD finite state machine: routes numeric input to menu pages.

All response pages are validated to be within USSD_MAX_CHARS (182).
Implements FR-M13-024 through FR-M13-030 (WP-4.7, M13).
"""

from __future__ import annotations

import logging

from .views import USSD_MAX_CHARS

_log_ = logging.getLogger(__name__)

# Menu text constants -- kept short to stay within 182-char budget
_MAIN_MENU = "EEL Verify\n1. Verify Token\n2. Results\n3. Grievance\n4. Exit"
_TOKEN_PART1 = "Enter first 8 digits of your ballot token:"
_TOKEN_PART2 = "Enter last 8 digits:"
_RESULTS_PROMPT = "Enter constituency code (e.g. 001):"
_GRIEVANCE_TYPE = "Select issue type:\n1. Violence\n2. Fraud\n3. Other"
_GRIEVANCE_CONFIRM = "Issue recorded. Confirm? 1=Yes 2=No"
_GRIEVANCE_DONE = "Grievance submitted. Ref: pending. Dial again to check status."
_EXIT_MSG = "Thank you for using EEL Verify."
_INVALID = "Invalid choice. Please try again."


def _assert_budget(text: str) -> None:
    """Raise if page exceeds 182-char USSD budget (FR-M13-029)."""
    if len(text) > USSD_MAX_CHARS:
        raise ValueError(
            f"USSD page exceeds {USSD_MAX_CHARS} chars (got {len(text)}): {text!r}"
        )


# Validate all static pages at import time
for _page in [
    _MAIN_MENU,
    _TOKEN_PART1,
    _TOKEN_PART2,
    _RESULTS_PROMPT,
    _GRIEVANCE_TYPE,
    _GRIEVANCE_CONFIRM,
    _GRIEVANCE_DONE,
    _EXIT_MSG,
    _INVALID,
]:
    _assert_budget(_page)


def parse_steps(text: str) -> list[str]:
    """Split USSD input text by '*' into ordered steps.

    Empty string maps to an empty list (first visit).
    """
    if not text:
        return []
    return text.split("*")


def _verify_token_result(part1: str, part2: str) -> str:
    """Stub: return inclusion status for the given token halves.

    In production this calls the BFF token endpoint via httpx.
    """
    token = f"{part1}{part2}".upper().strip()
    _log_.info("ussd.verify_token token_prefix=%s", token[:4])
    # Stub: tokens starting with 'VALID' are verified
    if token.startswith("VALID"):
        return f"Token {token[:4]}... VALID. Merkle proof confirmed."
    return f"Token {token[:4]}... NOT FOUND. Visit verify.eel.go.ke"


def _results_text(code: str) -> str:
    """Stub: return results summary for constituency code."""
    code = code.strip()
    text = f"Results {code}: Tallying in progress. Visit verify.eel.go.ke"
    _assert_budget(text)
    return text


def route_menu(steps: list[str]) -> tuple[str, bool]:
    """Map USSD step sequence to (response_text, is_final).

    Maximum depth: 5 steps (FR-M13-024).
    Returns (CON text, False) for intermediate pages.
    Returns (END text, True) for terminal pages.
    """
    depth = len(steps)

    if depth == 0:
        return _MAIN_MENU, False

    top = steps[0]

    if top == "1":
        # Verify token path
        if depth == 1:
            return _TOKEN_PART1, False
        if depth == 2:
            return _TOKEN_PART2, False
        if depth == 3:
            result = _verify_token_result(steps[1], steps[2])
            _assert_budget(result)
            return result, True

    elif top == "2":
        # Results lookup path
        if depth == 1:
            return _RESULTS_PROMPT, False
        if depth == 2:
            return _results_text(steps[1]), True

    elif top == "3":
        # Grievance path
        if depth == 1:
            return _GRIEVANCE_TYPE, False
        if depth == 2:
            if steps[1] in ("1", "2", "3"):
                return _GRIEVANCE_CONFIRM, False
            return _INVALID, False
        if depth == 3:
            if steps[2] == "1":
                return _GRIEVANCE_DONE, True
            return _EXIT_MSG, True

    elif top == "4":
        return _EXIT_MSG, True

    return _INVALID, False
