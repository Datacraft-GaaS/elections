import hashlib


def derive_iab_card_seed(card_serial: str, election_id: str, nist_value: str) -> str:
    """Derive audit seed from IAB card serial, election ID, and NIST beacon value."""
    combined = f"{card_serial}:{election_id}:{nist_value}"
    return hashlib.sha256(combined.encode()).hexdigest()
