"""Whistleblower profile management with encrypted-at-rest contact (WP-4.1)."""

from __future__ import annotations

import logging
from datetime import datetime, timezone

from cryptography.fernet import Fernet

from .models import WhistleblowerProfile

_log_ = logging.getLogger(__name__)


def uuid7str() -> str:
    from uuid6 import uuid7

    return str(uuid7())


class WhistleblowerService:
    """Manage whistleblower profiles with Fernet-encrypted contact data."""

    def __init__(self, fernet_key: bytes | None = None) -> None:
        # In production the key comes from eel-secrets. Stub: generate ephemeral.
        self._key = fernet_key or Fernet.generate_key()
        self._fernet = Fernet(self._key)

    async def register_profile(
        self,
        case_id: str,
        pseudonym: str,
        contact: str,
        protection_level: str = "standard",
    ) -> WhistleblowerProfile:
        assert case_id, "case_id required"
        assert pseudonym, "pseudonym required"
        assert protection_level in ("standard", "enhanced", "high"), (
            "invalid protection_level"
        )

        encrypted_contact = self._fernet.encrypt(contact.encode()) if contact else None
        profile = WhistleblowerProfile(
            profile_id=uuid7str(),
            case_id=case_id,
            pseudonym=pseudonym,
            protection_level=protection_level,  # type: ignore[arg-type]
            encrypted_contact=encrypted_contact,
            created_at=datetime.now(tz=timezone.utc),
        )
        _log_.info(
            "whistleblower.register profile_id=%s case_id=%s protection=%s",
            profile.profile_id,
            case_id,
            protection_level,
        )
        return profile

    async def decrypt_contact(
        self, profile: WhistleblowerProfile, key: bytes | None = None
    ) -> str:
        """Decrypt and return the contact string. Uses service key if no key provided."""
        assert profile.encrypted_contact is not None, "no encrypted_contact on profile"
        fernet = Fernet(key) if key else self._fernet
        return fernet.decrypt(profile.encrypted_contact).decode()
