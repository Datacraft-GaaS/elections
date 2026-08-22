"""Internal key derivation helper (not part of public API)."""

from __future__ import annotations

import hashlib


def _derive_key(seed: str, tag_epc: str) -> bytes:
    """Derive a 32-byte HMAC key from (seed, tag_epc) using SHA-256."""
    return hashlib.sha256(f"{seed}:{tag_epc}".encode()).digest()
