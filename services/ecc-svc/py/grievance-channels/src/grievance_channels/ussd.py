"""USSD/SMS ingestion handler for grievance intake (WP-4.1)."""

from __future__ import annotations

import logging

from .models import UssdSession

_log_ = logging.getLogger(__name__)

_MAX_USSD_LEN = 182

_MENU_STEP_0 = (
    "CON Uchaguzi Malalamiko\n1. Toa malalamiko\n2. Angalia hali ya kesi\n3. Toka"
)
_CATEGORIES = {
    "1": "intimidation",
    "2": "bribery",
    "3": "ballot_tampering",
    "4": "voter_suppression",
    "5": "other",
}


class UssdHandler:
    """Process USSD session state machine for grievance intake."""

    async def handle_session(
        self,
        session_id: str,
        msisdn: str,
        input_text: str,
        session: UssdSession | None = None,
    ) -> str:
        if session is None:
            session = UssdSession(session_id=session_id, phone_msisdn=msisdn)

        step = session.step
        inp = input_text.strip()

        if step == 0:
            session.step = 1
            response = _MENU_STEP_0
        elif step == 1:
            if inp == "1":
                session.step = 2
                response = (
                    "CON Chagua aina ya malalamiko:\n"
                    "1. Vitisho\n2. Rushwa\n3. Udanganyifu wa kura\n4. Kuzuia wapigakura\n5. Nyingine"
                )
            elif inp == "2":
                session.step = 10
                response = "CON Ingiza nambari ya kesi yako:"
            else:
                response = "END Asante. Tutawasiliana nawe."
        elif step == 2:
            category = _CATEGORIES.get(inp, "other")
            session.case_draft["category"] = category
            session.step = 3
            response = (
                "CON Kiwango cha dharura:\n1. Chini\n2. Wastani\n3. Juu\n4. Dharura"
            )
        elif step == 3:
            severity_map = {"1": "low", "2": "medium", "3": "high", "4": "critical"}
            session.case_draft["severity"] = severity_map.get(inp, "low")
            session.step = 4
            response = "CON Elezea malalamiko kwa ufupi (maneno 20):"
        elif step == 4:
            session.case_draft["description_snippet"] = inp[:100]
            session.case_draft["msisdn_hash"] = _hash_msisdn(msisdn)
            session.step = 99
            case_ref = session.case_draft.get(
                "case_ref", "GRV-" + session_id[-6:].upper()
            )
            response = f"END Malalamiko yako yamesajiliwa. Kumbukumbu: {case_ref}. Utapata ujumbe wa SMS."
        elif step == 10:
            case_ref = inp.strip()
            response = f"END Hali ya kesi {case_ref}: Inaendelea kushughulikiwa. Utaarifiwa hivi karibuni."
        else:
            response = "END Kosa la mfumo. Tafadhali jaribu tena."

        assert len(response) <= _MAX_USSD_LEN, (
            f"USSD response too long: {len(response)}"
        )
        _log_.info(
            "ussd.session session_id=%s step=%d->%d", session_id, step, session.step
        )
        return response


def _hash_msisdn(msisdn: str) -> str:
    import hashlib

    return hashlib.sha256(msisdn.encode()).hexdigest()
