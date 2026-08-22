"""Common LLRP-1.1 client shared by Zebra FX9600 and Impinj adapters.

Uses sllurp (Simple LLRP, MIT-licensed) as the primary LLRP library.
EPCglobal Low Level Reader Protocol 1.1 is a royalty-free open standard.

LLRP message flow:
  CONNECT -> SET_READER_CONFIG -> ADD_ROSPEC -> ENABLE_ROSPEC ->
  START_ROSPEC -> poll RO_ACCESS_REPORT -> (repeat)

EPC Gen2v2 Authenticate uses ACCESS_SPEC with OpSpec type 1024.
Falls back to Gen2v1 Inventory when the tag does not support Gen2v2 crypto.
"""

from __future__ import annotations

import asyncio
import enum
import logging
import time
from typing import Any

_log_ = logging.getLogger(__name__)

try:
    from sllurp import llrp as _sllurp  # type: ignore[import-untyped]

    _SLLURP_AVAILABLE = True
except ImportError:
    _sllurp = None  # type: ignore[assignment]
    _SLLURP_AVAILABLE = False


class _CBState(enum.Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class LlrpCommonClient:
    """Asyncio-safe LLRP-1.1 client with auto-reconnect and circuit breaker.

    Wraps sllurp via a thread executor so the Twisted event loop runs in a
    background thread while the caller lives in asyncio.  In production, use
    twisted.internet.asyncioreactor for native integration.

    Circuit-breaker defaults:
      - Opens after 5 consecutive failures.
      - Enters HALF_OPEN (one probe allowed) after 30 s cooldown.
      - Resets to CLOSED on any successful call.
    """

    _CB_FAILURE_THRESHOLD: int = 5
    _CB_COOLDOWN_SECS: float = 30.0

    def __init__(self, host: str, port: int = 5084, key_seed: str = "llrp") -> None:
        assert host, "host must be non-empty"
        assert 1 <= port <= 65535, f"invalid LLRP port: {port}"
        self._host = host
        self._port = port
        self._key_seed = key_seed
        self._client: Any | None = None
        self._lock: asyncio.Lock | None = None  # initialised lazily (event-loop-safe)
        self._cb_state: _CBState = _CBState.CLOSED
        self._cb_failures: int = 0
        self._cb_opened_at: float = 0.0

    # ------------------------------------------------------------------
    # Circuit-breaker helpers
    # ------------------------------------------------------------------

    def _cb_allow(self) -> bool:
        if self._cb_state == _CBState.CLOSED:
            return True
        if self._cb_state == _CBState.OPEN:
            if time.monotonic() - self._cb_opened_at >= self._CB_COOLDOWN_SECS:
                self._cb_state = _CBState.HALF_OPEN
                return True
            return False
        # HALF_OPEN: allow one probe through.
        return True

    def _cb_success(self) -> None:
        self._cb_failures = 0
        self._cb_state = _CBState.CLOSED

    def _cb_failure(self) -> None:
        self._cb_failures += 1
        if self._cb_failures >= self._CB_FAILURE_THRESHOLD:
            self._cb_state = _CBState.OPEN
            self._cb_opened_at = time.monotonic()
            _log_.warning(
                "llrp circuit breaker opened host=%s failures=%d",
                self._host,
                self._cb_failures,
            )

    def _get_lock(self) -> asyncio.Lock:
        if self._lock is None:
            self._lock = asyncio.Lock()
        return self._lock

    # ------------------------------------------------------------------
    # Connection management
    # ------------------------------------------------------------------

    async def connect(self) -> None:
        """Open LLRP connection and configure the reader for EPC Gen2 inventory."""
        if not _SLLURP_AVAILABLE:
            raise RuntimeError(
                "sllurp is not installed. "
                "Run: pip install sllurp  (MIT-licensed, no commercial SDK required). "
                "See integrations/rfid-vendors/zebra-llrp/README.md for details."
            )
        async with self._get_lock():
            if self._client is not None:
                return
            assert _sllurp is not None  # guarded by _SLLURP_AVAILABLE check above
            _sl = _sllurp  # local capture so pyright narrows inside lambda
            loop = asyncio.get_event_loop()
            cfg = _sl.LLRPReaderConfig({"host": self._host, "port": self._port})
            self._client = await loop.run_in_executor(
                None,
                lambda: _sl.LLRPClient(cfg),
            )
            _log_.info("llrp connected host=%s port=%d", self._host, self._port)

    async def ensure_connected(self) -> None:
        """Connect if not already connected; triggers auto-reconnect on stale client."""
        if self._client is None:
            await self.connect()

    async def close(self) -> None:
        """Disconnect and release LLRP resources."""
        async with self._get_lock():
            if self._client is not None:
                try:
                    loop = asyncio.get_event_loop()
                    await loop.run_in_executor(None, self._client.disconnect)
                except Exception:
                    pass
                finally:
                    self._client = None

    # ------------------------------------------------------------------
    # LLRP operations
    # ------------------------------------------------------------------

    async def inventory_epc(self, tag_epc: str) -> bytes | None:
        """Run EPC Gen2 Inventory and return raw RO_ACCESS_REPORT bytes.

        LLRP flow: ADD_ROSPEC -> ENABLE_ROSPEC -> START_ROSPEC ->
                   poll RO_ACCESS_REPORT -> STOP_ROSPEC.
        """
        if not self._cb_allow():
            raise RuntimeError(f"circuit breaker open for {self._host}")
        await self.ensure_connected()
        assert self._client is not None
        try:
            loop = asyncio.get_event_loop()
            _c = self._client
            report: bytes = await loop.run_in_executor(
                None, lambda: _c.startInventory()
            )
            self._cb_success()
            return report
        except Exception as exc:
            self._cb_failure()
            self._client = None  # force reconnect
            _log_.warning("llrp inventory failed host=%s: %s", self._host, exc)
            raise

    async def gen2v2_authenticate(self, tag_epc: str, challenge: bytes) -> bytes | None:
        """Send EPC Gen2v2 Authenticate via ACCESS_SPEC OpSpec 1024.

        Returns the tag response bytes on success.
        Returns None when the tag does not support Gen2v2 (falls back to Gen2v1).
        Raises RuntimeError on circuit-breaker rejection or network failure.

        OpSpec type 1024 encodes the Gen2v2 Authenticate command per
        EPCglobal EPC Tag Data Standard 2.0, section 8.3.
        """
        if not self._cb_allow():
            raise RuntimeError(f"circuit breaker open for {self._host}")
        await self.ensure_connected()
        assert self._client is not None
        try:
            loop = asyncio.get_event_loop()
            _c = self._client
            response: bytes = await loop.run_in_executor(
                None,
                lambda: _c.sendAccessSpec(
                    tag_epc=tag_epc,
                    op_spec_id=1024,
                    op_data=challenge,
                ),
            )
            self._cb_success()
            return response
        except AttributeError:
            # Tag does not support Gen2v2 — fall back to Gen2v1 Inventory.
            _log_.info("tag %s: Gen2v2 unsupported, falling back to Gen2v1", tag_epc)
            return None
        except Exception as exc:
            self._cb_failure()
            self._client = None
            _log_.warning(
                "llrp gen2v2 authenticate failed host=%s: %s", self._host, exc
            )
            raise

    # ------------------------------------------------------------------
    # Properties for introspection
    # ------------------------------------------------------------------

    @property
    def host(self) -> str:
        return self._host

    @property
    def port(self) -> int:
        return self._port

    @property
    def circuit_state(self) -> str:
        return self._cb_state.value

    @property
    def connected(self) -> bool:
        return self._client is not None
