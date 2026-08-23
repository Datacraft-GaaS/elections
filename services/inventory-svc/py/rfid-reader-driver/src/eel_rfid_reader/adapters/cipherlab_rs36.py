"""CipherLab RS36 adapter via BLE (bleak, MIT-licensed) + EPC Gen2 command set.

Replaces the former CipherLab commercial SDK with standard Android BLE APIs
accessed from Python via the bleak library (cross-platform, MIT-licensed).

The RS36 is Android-based and supports open BLE profiles.  Production
integration uses Flutter platform channels for on-device Android BLE; this
Python adapter targets the dev/test harness and desktop scenarios.

BLE profile used:
  Primary service: EPC Gen2 GATT profile
  Fallback: Bluetooth SPP (Serial Port Profile, UUID 0x1101)
  Battery status: BLE Battery Service (UUID 0x180F)

EPC Gen2 Inventory + Access commands are sent as GATT characteristic writes.
RO_ACCESS_REPORT-equivalent arrives as GATT notifications.
"""

from __future__ import annotations

import asyncio
import hmac as _hmac
import logging
from typing import Any

from .._internal_key import _derive_key
from ..adapter import RfidReaderAdapter
from ..hmac_gen2v2 import build_challenge, generate_nonce, record_nonce
from ..models import VerifyResult

_log_ = logging.getLogger(__name__)

# BLE service/characteristic UUIDs.
_EPC_GEN2_SERVICE_UUID = "0000a100-0000-1000-8000-00805f9b34fb"
_EPC_GEN2_CMD_CHAR_UUID = "0000a101-0000-1000-8000-00805f9b34fb"
_EPC_GEN2_NOTIFY_CHAR_UUID = "0000a102-0000-1000-8000-00805f9b34fb"
_BATTERY_SERVICE_UUID = "0000180f-0000-1000-8000-00805f9b34fb"
_BATTERY_LEVEL_CHAR_UUID = "00002a19-0000-1000-8000-00805f9b34fb"
_SPP_SERVICE_UUID = "00001101-0000-1000-8000-00805f9b34fb"

try:
    import bleak as _bleak  # type: ignore[import-untyped]
    from bleak import BleakClient, BleakScanner  # type: ignore[import-untyped]

    _BLEAK_AVAILABLE = True
except ImportError:
    _bleak = None  # type: ignore[assignment]
    BleakClient = None  # type: ignore[assignment,misc]
    BleakScanner = None  # type: ignore[assignment,misc]
    _BLEAK_AVAILABLE = False


class CipherLabRS36Adapter(RfidReaderAdapter):
    """CipherLab RS36 handheld adapter via BLE + EPC Gen2 GATT profile.

    Uses bleak (MIT-licensed) for cross-platform BLE access.
    No CipherLab commercial SDK required.

    Args:
            device_address: BLE MAC address or UUID of the RS36 device.
            transport: "ble" (default) or "spp" (Bluetooth Serial Port Profile fallback).
            key_seed: key derivation seed; keep consistent with issuing HSM.
    """

    def __init__(
        self,
        device_address: str,
        transport: str = "ble",
        key_seed: str = "cipherlab",
    ) -> None:
        assert transport in ("ble", "spp"), "transport must be 'ble' or 'spp'"
        self._device_address = device_address
        self._transport = transport
        self._key_seed = key_seed
        self._pending: dict[str, bytes] = {}
        self._client: Any | None = None
        self._notify_queue: asyncio.Queue[bytes] = asyncio.Queue()

    def _key(self, tag_epc: str) -> bytes:
        return _derive_key(self._key_seed, tag_epc)

    # ------------------------------------------------------------------
    # BLE connection management
    # ------------------------------------------------------------------

    async def _ensure_connected(self) -> None:
        if self._client is not None and self._client.is_connected:
            return
        if not _BLEAK_AVAILABLE:
            raise RuntimeError(
                "bleak is not installed. "
                "Run: pip install bleak  (MIT-licensed, no commercial SDK required). "
                "See integrations/rfid-vendors/cipherlab-ble/README.md for details."
            )
        _log_.info(
            "ble connecting to %s transport=%s", self._device_address, self._transport
        )
        assert BleakClient is not None  # guarded by _BLEAK_AVAILABLE check above
        self._client = BleakClient(self._device_address)
        await self._client.connect()
        # Subscribe to EPC Gen2 notification characteristic for tag reports.
        await self._client.start_notify(_EPC_GEN2_NOTIFY_CHAR_UUID, self._on_notify)
        _log_.info("ble connected to %s", self._device_address)

    def _on_notify(self, _: object, data: bytes) -> None:
        """Handle incoming GATT notification (RO_ACCESS_REPORT equivalent)."""
        self._notify_queue.put_nowait(data)

    async def scan_for_device(self, timeout: float = 10.0) -> str | None:
        """Scan for CipherLab RS36 advertising the EPC Gen2 GATT profile.

        Returns the device address string if found, or None on timeout.
        """
        if not _BLEAK_AVAILABLE:
            raise RuntimeError("bleak is not installed")
        assert BleakScanner is not None  # guarded by _BLEAK_AVAILABLE check above
        devices = await BleakScanner.discover(timeout=timeout)
        for device in devices:
            if device.name and "cipherlab" in device.name.lower():
                return device.address
            # Fall back to checking advertised services.
            uuids = getattr(device, "metadata", {}).get("uuids", [])
            if _EPC_GEN2_SERVICE_UUID in uuids or _SPP_SERVICE_UUID in uuids:
                return device.address
        return None

    async def read_battery_level(self) -> int | None:
        """Read battery level (0-100) via BLE Battery Service (UUID 0x180F).

        Returns None if the battery service is unavailable.
        """
        await self._ensure_connected()
        assert self._client is not None
        try:
            data = await self._client.read_gatt_char(_BATTERY_LEVEL_CHAR_UUID)
            return int(data[0])
        except Exception as exc:
            _log_.warning("battery read failed: %s", exc)
            return None

    # ------------------------------------------------------------------
    # EPC Gen2 command interface
    # ------------------------------------------------------------------

    async def _send_epc_command(self, cmd: bytes) -> bytes:
        """Write an EPC Gen2 command to the GATT characteristic and await response."""
        await self._ensure_connected()
        assert self._client is not None
        # Clear any stale notifications before sending.
        while not self._notify_queue.empty():
            self._notify_queue.get_nowait()
        await self._client.write_gatt_char(_EPC_GEN2_CMD_CHAR_UUID, cmd)
        # Wait for RO_ACCESS_REPORT notification (5 s timeout).
        try:
            response = await asyncio.wait_for(self._notify_queue.get(), timeout=5.0)
            return response
        except TimeoutError:
            _log_.warning(
                "no EPC Gen2 response from %s within 5 s", self._device_address
            )
            return b""

    # ------------------------------------------------------------------
    # RfidReaderAdapter interface
    # ------------------------------------------------------------------

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
                await self._client.disconnect()
            except Exception as exc:
                _log_.warning(
                    "ble disconnect failed for %s: %s", self._device_address, exc
                )
            finally:
                self._client = None
