"""IvrService: routes voice calls to voter-service flows."""

from __future__ import annotations

import logging

from .tts.builder import build_menu, build_ssml

_log_ = logging.getLogger(__name__)

_MAIN_MENU_OPTIONS: list[tuple[str, str]] = [
    ("1", "voter registration verification"),
    ("2", "polling station lookup"),
    ("0", "repeat this menu"),
]


class IvrService:
    """Routes Africa's Talking IVR sessions to voter-service flows."""

    _sessions: dict[str, str]

    def __init__(self) -> None:
        self._sessions = {}

    async def handle_call(
        self,
        session_id: str,
        phone_number: str,
        dtmf_digits: str | None,
    ) -> str:
        """Process a voice webhook call and return SSML/text.

        Args:
                session_id: Africa's Talking session identifier.
                phone_number: Caller's E.164 phone number.
                dtmf_digits: Digits pressed so far (None on first call).

        Returns:
                SSML string to be spoken to the caller.
        """
        assert session_id, "session_id required"
        assert phone_number, "phone_number required"

        _log_.info(
            "ivr.call session=%s phone=%s dtmf=%r",
            session_id,
            phone_number,
            dtmf_digits,
        )

        state = self._sessions.get(session_id, "main_menu")

        if not dtmf_digits or state == "main_menu":
            self._sessions[session_id] = "awaiting_selection"
            return build_menu(_MAIN_MENU_OPTIONS)

        return await self.route_session(session_id, dtmf_digits, phone_number)

    async def route_session(
        self,
        session_id: str,
        dtmf_digits: str,
        phone_number: str,
    ) -> str:
        """Route to the appropriate flow based on DTMF input."""
        digit = dtmf_digits.strip()[-1] if dtmf_digits.strip() else ""

        if digit == "1":
            self._sessions[session_id] = "verify"
            return build_ssml(
                "Voter verification. Please say or key in your national ID number.",
            )
        if digit == "2":
            self._sessions[session_id] = "station_lookup"
            return build_ssml(
                "Polling station lookup. Please say or key in your national ID number.",
            )
        if digit == "0":
            self._sessions[session_id] = "main_menu"
            return build_menu(_MAIN_MENU_OPTIONS)

        return build_ssml(
            "Invalid selection. "
            + " ".join(f"Press {d} for {desc}." for d, desc in _MAIN_MENU_OPTIONS)
        )
