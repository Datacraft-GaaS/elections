"""RfidReaderAdapter abstract base: issue_challenge + submit_response."""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import VerifyResult


class RfidReaderAdapter(ABC):
    """Adapter interface that each vendor implementation satisfies."""

    @abstractmethod
    async def issue_challenge(self, tag_epc: str) -> tuple[str, str]:
        """Return (nonce_hex, challenge_hex) for tag_epc.

        The challenge bytes are HMAC-SHA256(reader_key, nonce || epc_bytes).
        """
        ...

    @abstractmethod
    async def submit_response(
        self,
        tag_epc: str,
        response_bytes: bytes,
        tamper_indicator: bool,
    ) -> VerifyResult:
        """Verify a tag's response to a previously issued challenge.

        Args:
                tag_epc: EPC of the responding tag.
                response_bytes: raw bytes the tag returned.
                tamper_indicator: vendor-reported tamper bit (antenna-break / cover-lift).

        Returns:
                VerifyResult with challenge_valid and tamper_detected.
        """
        ...

    async def close(self) -> None:
        """Release vendor SDK resources. Override as needed."""
