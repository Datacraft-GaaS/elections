"""CRL (Certificate/Badge Revocation List) generation helpers."""

import logging
from datetime import datetime, timezone


_log = logging.getLogger(__name__)


def generate_crl() -> dict:
    """Return current CRL as {generated_at, revoked_serials}.

    Queries BadgeRevocationEntry (unmanaged, may not exist) and
    CredentialRevocationEntry independently so one missing table
    does not prevent the other from contributing to the CRL.
    """
    all_serials: list[str] = []

    try:
        from accreditation_app.models import BadgeRevocationEntry

        all_serials.extend(
            str(s)
            for s in BadgeRevocationEntry.objects.values_list("badge_serial", flat=True)
        )
    except Exception as exc:
        _log.warning("BadgeRevocationEntry unavailable, skipping badge serials: %s", exc)

    try:
        from accreditation_app.models import CredentialRevocationEntry

        all_serials.extend(
            str(s)
            for s in CredentialRevocationEntry.objects.values_list(
                "credential_serial", flat=True
            )
        )
    except Exception as exc:
        _log.warning(
            "CredentialRevocationEntry unavailable, skipping credential serials: %s",
            exc,
        )

    return {
        "generated_at": datetime.now(tz=timezone.utc).isoformat(),
        "revoked_serials": all_serials,
    }
