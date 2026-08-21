"""Badge PDF and QR helpers."""

from uuid6 import uuid7


def uuid7str() -> str:
    return str(uuid7())


def generate_badge_serial() -> str:
    """Generate a new unique badge serial (UUID7)."""
    return uuid7str()


def verify_badge(serial: str, crl: dict) -> str:
    """Verify badge status against CRL.

    Returns 'ACTIVE', 'REVOKED', or 'SUSPENDED'.
    """
    revoked_serials: list[str] = crl.get("revoked_serials", [])
    if serial in revoked_serials:
        return "REVOKED"

    try:
        from accreditation_app.models import MediaAccreditation

        accred = MediaAccreditation.objects.filter(badge_serial=serial).first()
        if accred is None:
            return "REVOKED"
        if accred.status == "SUSPENDED":
            return "SUSPENDED"
        if accred.status in ("ACTIVE", "BADGE_ISSUED"):
            return "ACTIVE"
        return "REVOKED"
    except Exception:
        return "ACTIVE"
