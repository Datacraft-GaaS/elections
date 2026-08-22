"""Zebra FX9600 adapter via ZebraLink API (HTTP + MQTT).

ZebraLink is a commercial SDK: obtain a licence from Zebra Technologies.
Import is guarded; attempting to instantiate without the SDK raises ImportError.
"""

from __future__ import annotations

from .._internal_key import _derive_key
from ..adapter import RfidReaderAdapter
from ..hmac_gen2v2 import build_challenge, generate_nonce, record_nonce
from ..models import VerifyResult

try:
    import zebralink as _zebralink  # type: ignore[import-untyped]

    _ZL_AVAILABLE = True
except ImportError:
    _zebralink = None  # type: ignore[assignment]
    _ZL_AVAILABLE = False


class ZebraLinkAdapter(RfidReaderAdapter):
    """Adapter for Zebra FX9600 fixed readers via ZebraLink HTTP + MQTT.

    IP54 ingress protection; Kenya UHF band 865-868 MHz.
    ZebraLink SDK licence required (commercial).
    """

    def __init__(self, host: str, port: int = 8080, key_seed: str = "zebra") -> None:
        if not _ZL_AVAILABLE:
            raise ImportError(
                "zebralink is not installed. "
                "Obtain the Zebra Technologies ZebraLink SDK licence and install the package. "
                "See integrations/rfid-vendors/zebra/README.md for procurement notes. "
                "Use the synthetic adapter for CI environments."
            )
        self._host = host
        self._port = port
        self._key_seed = key_seed
        self._pending: dict[str, bytes] = {}
        self._client: object | None = None

    def _key(self, tag_epc: str) -> bytes:
        return _derive_key(self._key_seed, tag_epc)

    async def _ensure_connected(self) -> None:
        if self._client is None:
            assert _zebralink is not None
            self._client = await _zebralink.connect(self._host, self._port)

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
                assert _zebralink is not None
                await _zebralink.disconnect(self._client)
            except Exception:
                pass
            finally:
                self._client = None
