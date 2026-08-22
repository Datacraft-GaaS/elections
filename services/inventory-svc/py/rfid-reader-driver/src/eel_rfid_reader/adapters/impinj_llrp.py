"""Impinj Speedway R700 adapter via LLRP (EPCglobal 1.1).

Requires the 'llrpyc' package (not on PyPI by default; install from vendor).
Import is guarded: if llrpyc is absent, attempting to instantiate this adapter
raises ImportError with a clear message.
"""

from __future__ import annotations

from .._internal_key import _derive_key
from ..adapter import RfidReaderAdapter
from ..hmac_gen2v2 import build_challenge, generate_nonce, record_nonce
from ..models import VerifyResult

try:
    import llrpyc as _llrpyc  # type: ignore[import-untyped]

    _LLRP_AVAILABLE = True
except ImportError:
    _llrpyc = None  # type: ignore[assignment]
    _LLRP_AVAILABLE = False


class ImpinjLlrpAdapter(RfidReaderAdapter):
    """Adapter for Impinj Speedway R700 readers via LLRP 1.1.

    Kenya UHF band: 865-868 MHz (CAK 2022 band plan, EPC Gen2v2).
    Capacity: up to 10 tags/second per reader.
    """

    def __init__(self, host: str, port: int = 5084, key_seed: str = "impinj") -> None:
        if not _LLRP_AVAILABLE:
            raise ImportError(
                "llrpyc is not installed. "
                "Install it from the Impinj vendor SDK or use the synthetic adapter for CI. "
                "See integrations/rfid-vendors/impinj/README.md for procurement details."
            )
        self._host = host
        self._port = port
        self._key_seed = key_seed
        self._pending: dict[str, bytes] = {}
        # llrpyc client is created lazily on first use.
        self._client: object | None = None

    def _key(self, tag_epc: str) -> bytes:
        return _derive_key(self._key_seed, tag_epc)

    async def _ensure_connected(self) -> None:
        if self._client is None:
            assert _llrpyc is not None
            self._client = _llrpyc.LLRPClient(self._host, self._port)

    async def issue_challenge(self, tag_epc: str) -> tuple[str, str]:
        await self._ensure_connected()
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
        import hmac as _hmac

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
        valid = _hmac.compare_digest(expected, response_bytes)
        return VerifyResult(
            tag_epc=tag_epc,
            challenge_valid=valid,
            tamper_detected=tamper_indicator,
            message="ok" if valid else "hmac mismatch",
        )

    async def close(self) -> None:
        if self._client is not None:
            try:
                assert _llrpyc is not None
                await _llrpyc.close(self._client)
            except Exception:
                pass
            finally:
                self._client = None
