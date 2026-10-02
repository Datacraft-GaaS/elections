# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Ed25519 signing and verification for eel envelopes."""

from __future__ import annotations

import base64
import logging
from typing import Protocol

import nacl.exceptions
import nacl.signing

from eel_events.envelope import Envelope

logger = logging.getLogger(__name__)


class SpiffeKeyProvider(Protocol):
    async def get_signing_key(self) -> tuple[bytes, str]:
        """Return (private_key_bytes, key_id) where private_key_bytes is 32-byte Ed25519 seed."""
        ...


def sign_envelope(envelope: Envelope, private_key: bytes) -> str:
    """Sign the canonical bytes of an envelope with Ed25519.

    Args:
            envelope: Envelope to sign (eel_signature field is excluded from signed bytes).
            private_key: 32-byte Ed25519 seed.

    Returns:
            Signature string in the form "ed25519:<base64url>".
    """
    signing_key = nacl.signing.SigningKey(private_key)
    msg = envelope.canonical_bytes()
    signed = signing_key.sign(msg)
    sig_b64 = base64.urlsafe_b64encode(signed.signature).decode()
    return f"ed25519:{sig_b64}"


def verify_envelope(envelope: Envelope, public_key: bytes) -> bool:
    """Verify the eel_signature on an envelope.

    Args:
            envelope: Envelope with eel_signature populated.
            public_key: 32-byte Ed25519 verify key.

    Returns:
            True if signature is valid, False otherwise.
    """
    if not envelope.eel_signature:
        logger.warning(
            "verify_envelope called on envelope with no signature (id=%s)", envelope.id
        )
        return False

    try:
        prefix, sig_b64 = envelope.eel_signature.split(":", 1)
    except ValueError:
        logger.warning("Malformed eel_signature on envelope id=%s", envelope.id)
        return False

    if prefix != "ed25519":
        logger.warning(
            "Unknown signature algorithm %r on envelope id=%s", prefix, envelope.id
        )
        return False

    try:
        sig = base64.urlsafe_b64decode(sig_b64 + "==")  # pad for safety
    except Exception:
        return False

    verify_key = nacl.signing.VerifyKey(public_key)
    msg = envelope.canonical_bytes()
    try:
        verify_key.verify(msg, sig)
        return True
    except nacl.exceptions.BadSignatureError:
        return False
