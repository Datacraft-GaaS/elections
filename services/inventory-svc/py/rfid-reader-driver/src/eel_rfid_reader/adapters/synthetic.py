"""Hash-based synthetic adapter for CI and tests.

Deterministic: given the same (tag_epc, seed), always produces the same key and
challenge/response pair. Simulates tamper based on a configurable probability.
"""

from __future__ import annotations

import hashlib
import hmac
import random

from .._internal_key import _derive_key
from ..adapter import RfidReaderAdapter
from ..hmac_gen2v2 import build_challenge, generate_nonce, record_nonce
from ..models import VerifyResult


class SyntheticAdapter(RfidReaderAdapter):
    """Deterministic HMAC adapter: no hardware required.

    Config:
            seed: base seed for key derivation (default "synthetic").
            tamper_prob: probability [0.0, 1.0] that submit_response reports tamper.
    """

    def __init__(self, seed: str = "synthetic", tamper_prob: float = 0.0) -> None:
        assert 0.0 <= tamper_prob <= 1.0
        self._seed = seed
        self._tamper_prob = tamper_prob
        # Per-tag in-flight nonces: {tag_epc: nonce_bytes}
        self._pending: dict[str, bytes] = {}

    def _key(self, tag_epc: str) -> bytes:
        """Derive a deterministic 32-byte key for tag_epc from seed."""
        return _derive_key(self._seed, tag_epc)

    async def issue_challenge(self, tag_epc: str) -> tuple[str, str]:
        nonce = generate_nonce()
        key = self._key(tag_epc)
        challenge = build_challenge(key, nonce, tag_epc)
        nonce_hex = nonce.hex()
        record_nonce(tag_epc, nonce_hex)
        self._pending[tag_epc] = nonce
        return nonce_hex, challenge.hex()

    async def submit_response(
        self,
        tag_epc: str,
        response_bytes: bytes,
        tamper_indicator: bool,
    ) -> VerifyResult:
        nonce = self._pending.pop(tag_epc, None)
        if nonce is None:
            return VerifyResult(
                tag_epc=tag_epc,
                challenge_valid=False,
                tamper_detected=tamper_indicator,
                message="no pending challenge for tag",
            )
        key = self._key(tag_epc)
        expected = build_challenge(key, nonce, tag_epc)
        valid = hmac.compare_digest(expected, response_bytes)
        simulated_tamper = random.random() < self._tamper_prob
        return VerifyResult(
            tag_epc=tag_epc,
            challenge_valid=valid,
            tamper_detected=tamper_indicator or simulated_tamper,
            message="ok" if valid else "hmac mismatch",
        )
