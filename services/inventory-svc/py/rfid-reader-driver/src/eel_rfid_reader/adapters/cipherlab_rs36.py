"""CipherLab RS36 handheld adapter via BLE + serial.

CipherLab SDK is a commercial dependency; import is guarded with a clear error.
The RS36 is an IP65-rated handheld PDA enrolled via Android Enterprise MDM.
"""

from __future__ import annotations

from .._internal_key import _derive_key
from ..adapter import RfidReaderAdapter
from ..hmac_gen2v2 import build_challenge, generate_nonce, record_nonce
from ..models import VerifyResult

try:
    import cipherlab_sdk as _cl  # type: ignore[import-untyped]

    _CL_AVAILABLE = True
except ImportError:
    _cl = None  # type: ignore[assignment]
    _CL_AVAILABLE = False


class CipherLabRS36Adapter(RfidReaderAdapter):
    """Adapter for CipherLab RS36 handheld via BLE / USB-serial.

    IP65 ingress protection; Android Enterprise MDM enrollment required.
    Use the synthetic adapter in CI; this class requires the CipherLab SDK.
    """

    def __init__(
        self,
        device_address: str,
        transport: str = "ble",
        key_seed: str = "cipherlab",
    ) -> None:
        if not _CL_AVAILABLE:
            raise ImportError(
                "cipherlab_sdk is not installed. "
                "Obtain the CipherLab RS36 SDK from CipherLab and install it. "
                "See integrations/rfid-vendors/cipherlab/README.md for procurement notes. "
                "Use the synthetic adapter for CI environments."
            )
        assert transport in ("ble", "serial"), "transport must be 'ble' or 'serial'"
        self._device_address = device_address
        self._transport = transport
        self._key_seed = key_seed
        self._pending: dict[str, bytes] = {}
        self._connection: object | None = None

    def _key(self, tag_epc: str) -> bytes:
        return _derive_key(self._key_seed, tag_epc)

    async def _ensure_connected(self) -> None:
        if self._connection is None:
            assert _cl is not None
            self._connection = await _cl.connect(
                self._device_address, transport=self._transport
            )

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
        if self._connection is not None:
            try:
                assert _cl is not None
                await _cl.disconnect(self._connection)
            except Exception:
                pass
            finally:
                self._connection = None
